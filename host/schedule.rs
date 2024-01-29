use crate::*;
use chrono::TimeZone;
use std::future::Future;
use tokio::time::sleep;
pub use tokio_schedule::Job;
use tokio_schedule::{every, Every};

pub struct Schedule {
    pub runtime: Runtime,
}

impl Schedule {
    pub fn every(period: u32) -> Every {
        every(period)
    }
}

state!(SCHEDULE: Schedule = { Schedule { runtime: Runtime::new().unwrap() } });

pub trait Schedulable: Job {
    /// This method returns Future that cyclic performs the job
    fn spawn<'a, F, Fut>(self, func: F)
    where
        Self: Send + 'static,
        F: FnMut() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'a,
        <Self::TZ as TimeZone>::Offset: Send + 'a;
}

impl<T: Job> Schedulable for T {
    fn spawn<'a, F, Fut>(self, mut func: F)
    where
        Self: Send + 'static,
        F: FnMut() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'a,
        <Self::TZ as TimeZone>::Offset: Send + 'a,
    {
        SCHEDULE.runtime.spawn(async move {
            while let Some(dur) = self.time_to_sleep() {
                sleep(dur).await;
                func().await;
            }
        });
    }
}
