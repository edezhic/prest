use crate::*;
use chrono::TimeZone;
use std::{future::Future, sync::atomic::{Ordering, AtomicUsize}};
use tokio::time::sleep;
pub use tokio_schedule::Job as RepeatableJob;
use tokio_schedule::{every, Every};

pub struct PrestRuntime {
    pub inner: Runtime,
    pub running_scheduled_tasks: AtomicUsize,
}

impl std::ops::Deref for PrestRuntime {
    type Target = Runtime;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl PrestRuntime {
    pub fn every(&self, period: u32) -> Every {
        every(period)
    }

    pub fn once<'a, Fut>(&self, fut: Fut)
    where
        Self: Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        RT.spawn(async move {
            RT.running_scheduled_tasks.fetch_add(1, Ordering::SeqCst);
            fut.await;
            let current_tasks = RT.running_scheduled_tasks.fetch_sub(1, Ordering::SeqCst);
            SHUTDOWN
                .scheduled_task_running
                .store(current_tasks == 0, Ordering::SeqCst);
        });
    }

    pub fn try_once<'a, Fut>(&self, fut: Fut)
    where
        Self: Send + 'static,
        Fut: Future<Output = Result> + Send + 'static,
    {
        RT.spawn(async move {
            RT.running_scheduled_tasks.fetch_add(1, Ordering::SeqCst);
            if let Err(e) =fut.await {
                error!("{e}");
            }
            let current_tasks = RT.running_scheduled_tasks.fetch_sub(1, Ordering::SeqCst);
            SHUTDOWN
                .scheduled_task_running
                .store(current_tasks == 0, Ordering::SeqCst);
        });
    }
}

/// Simplified interface to run [`RepeatableJob`]s in prest's [`RT`]
pub trait Schedulable: RepeatableJob {
    /// This method returns Future that cyclic performs the job
    fn spawn<'a, F, Fut>(self, func: F)
    where
        Self: Send + 'static,
        F: FnMut() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'a,
        <Self::TZ as TimeZone>::Offset: Send + 'a;
}

impl<T: RepeatableJob> Schedulable for T {
    fn spawn<'a, F, Fut>(self, mut func: F)
    where
        Self: Send + 'static,
        F: FnMut() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'a,
        <Self::TZ as TimeZone>::Offset: Send + 'a,
    {
        RT.spawn(async move {
            while let Some(dur) = self.time_to_sleep() {
                if SHUTDOWN.in_progress() {
                    break;
                }
                sleep(dur).await;
                if SHUTDOWN.in_progress() {
                    break;
                }
                RT.running_scheduled_tasks.fetch_add(1, Ordering::SeqCst);
                func().await;
                let current_tasks = RT.running_scheduled_tasks.fetch_sub(1, Ordering::SeqCst);
                SHUTDOWN
                    .scheduled_task_running
                    .store(current_tasks == 0, Ordering::SeqCst);
            }
        });
    }
}
