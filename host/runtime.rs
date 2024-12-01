use crate::*;
use axum_server::Handle;
use core::{
    pin::Pin,
    task::{ready, Context, Poll},
};
use pin_project_lite::pin_project;
use std::{
    boxed::Box,
    future::Future,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};
use tokio::time::sleep;
pub use tokio_schedule::Job as RepeatableJob;
use tokio_schedule::{every, Every};
use tracing::{trace_span as span, Span};

pub struct PrestRuntime {
    pub inner: Runtime,
    pub running_scheduled_tasks: AtomicUsize,
    pub shutting_down: AtomicBool,
    pub server_handles: std::sync::RwLock<Vec<Handle>>,
}

impl PrestRuntime {
    pub fn init() -> Self {
        let inner = Runtime::new().unwrap();
        #[cfg(unix)]
        inner.spawn(async { RT.listen_shutdown() });
        PrestRuntime {
            inner,
            running_scheduled_tasks: 0.into(),
            shutting_down: false.into(),
            server_handles: Default::default(),
        }
    }

    pub fn every(&self, period: u32) -> Every {
        every(period)
    }

    pub fn once<Fut>(&self, fut: Fut)
    where
        Self: Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        RT.spawn(async {
            ScheduledJobFuture::from(fut, span!("once job")).await;
        });
    }

    pub fn try_once<Fut>(&self, fut: Fut)
    where
        Self: Send + 'static,
        Fut: Future<Output = Result> + Send + 'static,
    {
        RT.spawn(async {
            if let Err(e) = ScheduledJobFuture::from(fut, span!("try once job")).await {
                error!("{e}");
            }
        });
    }

    pub fn shutdown(&self) {
        if self.shutting_down() {
            return;
        } else {
            self.shutting_down.store(true, Ordering::SeqCst);
        }
        // stopping the servers
        for handle in self.server_handles.read().unwrap().iter() {
            handle.graceful_shutdown(Some(std::time::Duration::from_secs(1)))
        }
        debug!("Sent graceful shutdown signals for servers");

        // awaiting currently running scheduled tasks
        while RT.running_scheduled_tasks.load(Ordering::SeqCst) > 0 {
            std::thread::sleep(std::time::Duration::from_millis(10));
            continue;
        }
        debug!("Awaited scheduled tasks completion");

        // flushing dirty db writes
        #[cfg(feature = "db")]
        DB.flush();
        debug!("Flushed the DB");

        warn!("Finished shutdown procedures");
    }

    pub fn shutting_down(&self) -> bool {
        self.shutting_down.load(Ordering::SeqCst)
    }

    pub fn new_server_handle(&self) -> Handle {
        let handle = Handle::new();
        self.server_handles.write().unwrap().push(handle.clone());
        handle
    }

    #[cfg(unix)]
    pub async fn listen_shutdown(&self) {
        match tokio_signal::unix::signal(tokio_signal::unix::SignalKind::terminate()) {
            Ok(mut sigterm) => {
                sigterm.recv().await;
                warn!("Received shutdown(SIGTERM) signal, initiating");
                RT.shutdown();
            }
            Err(err) => {
                error!("Error listening for shutdown(SIGTERM) signal: {}", err);
            }
        }
    }
}

impl std::ops::Deref for PrestRuntime {
    type Target = Runtime;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Describes collected stats for scheduled jobs
#[derive(Debug, Table, Clone, Serialize, Deserialize)]
pub struct ScheduledJobRecord {
    pub id: Uuid,
    pub name: String,
    pub start: NaiveDateTime,
    pub end: Option<NaiveDateTime>,
    pub error: Option<String>,
}

impl ScheduledJobRecord {
    pub fn start(name: &str) -> Self {
        let stat = ScheduledJobRecord {
            id: Uuid::now_v7(),
            name: name.to_owned(),
            start: Utc::now().naive_utc(),
            end: None,
            error: None,
        };
        trace!("Starting job {name} at {}", stat.start);
        let stat_clone = stat.clone();
        RT.spawn_blocking(move || {
            if let Err(e) = stat_clone.save() {
                error!("Failed to record start of the scheduled job stat {stat_clone:?} : {e}");
            }
        });
        stat
    }

    pub fn end(mut self, error: Option<String>) {
        let end = Utc::now().naive_utc();

        trace!("Ended job {} at {end}", self.name);

        if let Err(e) = self.update_end(Some(end)) {
            error!("Failed to record end of the scheduled job stat {self:?} : {e}");
        }

        if let Some(e) = error {
            error!("Scheduled job {} error: {e}", self.name);
            if let Err(upd_e) = self.update_error(Some(e.clone())) {
                error!("Failed to record error {e} of the scheduled job stat {self:?} : {upd_e}");
            }
        }
    }
}

/// Simplified interface to run [`RepeatableJob`]s in prest's [`RT`]
pub trait Schedulable<O>: RepeatableJob {
    /// This method spawns the Future in cycle (and logs errors if any)
    fn spawn<'a, F, Fut>(self, func: F)
    where
        Self: Send + 'static,
        F: FnMut() -> Fut + Send + 'static,
        Fut: Future<Output = O> + Send + 'a;

    /// This method spawns the Future in cycle and records performance stats
    fn schedule<'a, F, Fut>(self, job_name: &'static str, func: F)
    where
        Self: Send + 'static,
        F: FnMut() -> Fut + Send + 'static,
        Fut: Future<Output = O> + Send + 'a;
}

impl<T: RepeatableJob> Schedulable<()> for T {
    fn spawn<'a, F, Fut>(self, mut func: F)
    where
        Self: Send + 'static,
        F: FnMut() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'a,
    {
        RT.spawn(async move {
            while self.should_proceed().await {
                ScheduledJobFuture::from(func(), span!("repeatable job")).await;
            }
        });
    }

    fn schedule<'a, F, Fut>(self, job_name: &'static str, mut func: F)
    where
        Self: Send + 'static,
        F: FnMut() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'a,
    {
        RT.spawn(async move {
            while self.should_proceed().await {
                let stat = ScheduledJobRecord::start(job_name);
                ScheduledJobFuture::from(func(), span!("repeatable job", job = job_name)).await;
                stat.end(None);
            }
        });
    }
}

impl<T: RepeatableJob, E: std::fmt::Display + 'static> Schedulable<Result<(), E>> for T {
    fn spawn<'a, F, Fut>(self, mut func: F)
    where
        Self: Send + 'static,
        F: FnMut() -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), E>> + Send + 'a,
    {
        RT.spawn(async move {
            while self.should_proceed().await {
                if let Err(e) = ScheduledJobFuture::from(func(), span!("repeatable job")).await {
                    error!("Repeatable job error: {e}");
                }
            }
        });
    }

    fn schedule<'a, F, Fut>(self, job_name: &'static str, mut func: F)
    where
        Self: Send + 'static,
        F: FnMut() -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), E>> + Send + 'a,
    {
        RT.spawn(async move {
            while self.should_proceed().await {
                let stat = ScheduledJobRecord::start(job_name);
                let err = ScheduledJobFuture::from(func(), span!("repeatable job", job = job_name))
                    .await
                    .err()
                    .map(|e| e.to_string());
                stat.end(err);
            }
        });
    }
}

pub(crate) trait ShouldProceed: RepeatableJob {
    async fn should_proceed(&self) -> bool;
}
impl<T: RepeatableJob> ShouldProceed for T {
    async fn should_proceed(&self) -> bool {
        if RT.shutting_down() {
            return false;
        }
        let Some(duration) = self.time_to_sleep() else {
            return false;
        };
        sleep(duration).await;
        if RT.shutting_down() {
            return false;
        }
        true
    }
}

pin_project! {
    struct ScheduledJobFuture<F: Future> {
        #[pin]
        pub(crate) inner: F,
        pub(crate) span: Span,
    }
}
impl<F: Future> ScheduledJobFuture<F> {
    pub fn from(inner: F, span: Span) -> Self {
        RT.running_scheduled_tasks.fetch_add(1, Ordering::SeqCst);
        Self { inner, span }
    }
}
impl<Fut, O> Future for ScheduledJobFuture<Fut>
where
    Fut: Future<Output = O>,
{
    type Output = O;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _guard = this.span.enter();
        let output = ready!(this.inner.poll(cx));
        RT.running_scheduled_tasks.fetch_sub(1, Ordering::SeqCst);
        Poll::Ready(output)
    }
}
