use crate::*;
use chrono::TimeZone;
use std::{
    future::Future,
    sync::atomic::{AtomicUsize, Ordering},
};
use tokio::time::sleep;
pub use tokio_schedule::Job as RepeatableJob;
use tokio_schedule::{every, Every};


state!(SCHEDULE: Schedule = { Schedule { runtime: Runtime::new().unwrap(), running_tasks: 0.into() } });

pub struct Schedule {
    pub runtime: Runtime,
    pub running_tasks: AtomicUsize,
}

impl Schedule {
    pub fn every(&self, period: u32) -> Every {
        every(period)
    }

    pub fn once<'a, F, Fut>(&self, func: F)
    where
        Self: Send + 'static,
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'a,
    {
        SCHEDULE.runtime.spawn(async move {
            SCHEDULE.running_tasks.fetch_add(1, Ordering::SeqCst);
            func().await;
            let current_tasks = SCHEDULE.running_tasks.fetch_sub(1, Ordering::SeqCst);
            SHUTDOWN
                .scheduled_task_running
                .store(current_tasks == 0, Ordering::SeqCst);
        });
    }
}


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
        SCHEDULE.runtime.spawn(async move {
            while let Some(dur) = self.time_to_sleep() {
                if SHUTDOWN.in_progress() {
                    break;
                }
                sleep(dur).await;
                if SHUTDOWN.in_progress() {
                    break;
                }
                SCHEDULE.running_tasks.fetch_add(1, Ordering::SeqCst);
                func().await;
                let current_tasks = SCHEDULE.running_tasks.fetch_sub(1, Ordering::SeqCst);
                SHUTDOWN
                    .scheduled_task_running
                    .store(current_tasks == 0, Ordering::SeqCst);
            }
        });
    }
}
