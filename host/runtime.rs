use crate::*;
use axum_server::Handle;
use core::{
    pin::Pin,
    task::{ready, Context, Poll},
};
use host::get_panic_message;
use pin_project_lite::pin_project;
use std::{
    boxed::Box,
    future::Future,
    panic::AssertUnwindSafe,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};
use tokio::runtime::Runtime;
#[doc(hidden)]
pub use tokio_schedule::Job as RepeatableJob;
use tokio_schedule::{every, Every};
use tracing::{trace_span as span, Span};

/// Wrapper around [`tokio::runtime::Runtime`] that manages schedule and graceful shutdown
pub struct PrestRuntime {
    pub inner: Runtime,
    pub running_scheduled_tasks: AtomicUsize,
    pub ready: AtomicBool,
    pub shutting_down: AtomicBool,
    pub server_handles: std::sync::RwLock<Vec<Handle>>,
}

impl PrestRuntime {
    pub fn init() -> Self {
        let inner = Runtime::new().expect("Prest should be able to initialize inner tokio runtime");
        #[cfg(unix)]
        inner.spawn(async { RT.listen_shutdown() });
        PrestRuntime {
            inner,
            running_scheduled_tasks: 0.into(),
            ready: false.into(),
            shutting_down: false.into(),
            server_handles: Default::default(),
        }
    }

    pub fn ready(&self) -> bool {
        self.ready.load(Ordering::SeqCst)
    }

    pub fn set_ready(&self) {
        self.ready.store(true, Ordering::SeqCst);
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
            if let Err(e) = AssertUnwindSafe(ScheduledJobFuture::from(fut, span!("once job")))
                .catch_unwind()
                .await
            {
                error!(target:"runtime", "Panicked in `once` job: {}", get_panic_message(e));
            }
        });
    }

    pub fn try_once<Fut>(&self, fut: Fut)
    where
        Self: Send + 'static,
        Fut: Future<Output = Result> + Send + 'static,
    {
        RT.spawn(async {
            match AssertUnwindSafe(ScheduledJobFuture::from(fut, span!("try once job")))
                .catch_unwind()
                .await
            {
                Err(e) => error!(target:"runtime", "Panicked in `try_once` with: {}", get_panic_message(e)),
                Ok(Err(e)) => error!(target:"runtime", "{e}"),
                Ok(Ok(())) => (),
            };
        });
    }

    pub async fn shutdown(&self) {
        if self.shutting_down() {
            return;
        } else {
            self.shutting_down.store(true, Ordering::SeqCst);
        }
        // stopping the servers
        for handle in self.server_handles.read().unwrap().iter() {
            handle.graceful_shutdown(Some(std::time::Duration::from_secs(1)))
        }
        debug!(target:"runtime", "Sent graceful shutdown signals for servers");

        // awaiting currently running scheduled tasks
        while RT.running_scheduled_tasks.load(Ordering::SeqCst) > 0 {
            sleep(std::time::Duration::from_millis(10)).await;
            continue;
        }
        debug!(target:"runtime", "Awaited scheduled tasks completion");

        // flushing dirty db writes
        #[cfg(feature = "db")]
        DB.flush().await;
        debug!(target:"runtime", "Flushed the DB");

        warn!(target:"runtime", "Finished shutdown procedures");
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
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sigterm) => {
                sigterm.recv().await;
                warn!(target:"runtime", "Received shutdown(SIGTERM) signal, initiating");
                RT.shutdown().await;
            }
            Err(err) => {
                error!(target:"runtime", "Error listening for shutdown(SIGTERM) signal: {}", err);
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
        trace!(target:"runtime", job = %name, start = %stat.start);
        let stat_clone = stat.clone();
        RT.spawn(async move {
            if let Err(e) = stat_clone.save().await {
                error!(target:"runtime", "Failed to record start of the scheduled job stat {stat_clone:?} : {e}");
            }
        });
        stat
    }

    pub async fn end(mut self, error: Option<String>) {
        let end = Utc::now().naive_utc();

        trace!(target:"runtime", job = %self.name, end = %end);

        if let Err(e) = self.update_end(Some(end)).await {
            error!(target:"runtime", "Failed to record end of the scheduled job stat {self:?} : {e}");
        }

        if let Some(e) = error {
            error!(target:"runtime", "Scheduled job {} error: {e}", self.name);
            if let Err(upd_e) = self.update_error(Some(e.clone())).await {
                error!(target:"runtime", "Failed to record error {e} of the scheduled job stat {self:?} : {upd_e}");
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
                if let Err(e) =
                    AssertUnwindSafe(ScheduledJobFuture::from(func(), span!("repeatable job")))
                        .catch_unwind()
                        .await
                {
                    error!(target:"runtime", "Panicked in repeatable job: {}", get_panic_message(e));
                }
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
                if let Err(e) = AssertUnwindSafe(ScheduledJobFuture::from(
                    func(),
                    span!("repeatable job", job = job_name),
                ))
                .catch_unwind()
                .await
                {
                    error!(
                        target:"runtime",
                        "Panicked in scheduled job {job_name}: {}",
                        get_panic_message(e)
                    );
                }
                stat.end(None).await;
            }
        });
    }
}

impl<T: RepeatableJob, E: std::fmt::Display + 'static + Send> Schedulable<Result<(), E>> for T {
    fn spawn<'a, F, Fut>(self, mut func: F)
    where
        Self: Send + 'static,
        F: FnMut() -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), E>> + Send + 'a,
    {
        RT.spawn(async move {
            while self.should_proceed().await {
                match AssertUnwindSafe(ScheduledJobFuture::from(func(), span!("repeatable job")))
                    .catch_unwind()
                    .await
                {
                    Err(e) => error!(target:"runtime", "Panicked in repeatable job with: {}", get_panic_message(e)),
                    Ok(Err(e)) => error!(target:"runtime", "Repeatable job error: {e}"),
                    Ok(Ok(())) => (),
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
                match AssertUnwindSafe(ScheduledJobFuture::from(
                    func(),
                    span!("repeatable job", job = job_name),
                ))
                .catch_unwind()
                .await
                {
                    Err(e) => error!(
                        target:"runtime",
                        "Panicked in scheduled job {job_name} with: {}",
                        get_panic_message(e)
                    ),
                    Ok(Err(e)) => stat.end(Some(e.to_string())).await,
                    Ok(Ok(())) => stat.end(None).await,
                }
            }
        });
    }
}

pub(crate) trait ShouldProceed: RepeatableJob {
    async fn should_proceed(&self) -> bool;
}
impl<T: RepeatableJob> ShouldProceed for T {
    async fn should_proceed(&self) -> bool {
        while !RT.ready() {
            sleep(std::time::Duration::from_millis(1)).await;
        }
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

impl<F: Future + Send> ScheduledJobFuture<F> {
    pub fn from(inner: F, span: Span) -> Self {
        RT.running_scheduled_tasks.fetch_add(1, Ordering::SeqCst);
        Self { inner, span }
    }
}
impl<Fut, O> Future for ScheduledJobFuture<Fut>
where
    Fut: Future<Output = O> + Send,
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
