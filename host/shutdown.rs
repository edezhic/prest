use std::sync::atomic::{AtomicBool, Ordering};

use axum_server::Handle;

use crate::*;

state!(SHUTDOWN: Shutdown = { Shutdown::default() });

/// Interface for graceful shutdowns
#[derive(Debug, Default)]
pub struct Shutdown {
    pub initiated: AtomicBool,
    pub server_handles: std::sync::RwLock<Vec<Handle>>,
    pub scheduled_task_running: AtomicBool,
}

impl Shutdown {
    pub fn initiate(&self) {
        if self.in_progress() {
            return;
        } else {
            tracing::warn!("Initiating shutdown process");
            self.initiated.store(true, Ordering::SeqCst);
        }
        // stopping the servers
        for handle in self.server_handles.read().unwrap().iter() {
            handle.graceful_shutdown(Some(std::time::Duration::from_secs(1)))
        }
        // awaiting currently running scheduled tasks
        while self.scheduled_task_running.load(Ordering::SeqCst) {
            continue;
        }
        // flushing dirty db writes
        #[cfg(feature = "db")]
        DB.flush();
    }

    pub fn in_progress(&self) -> bool {
        self.initiated.load(Ordering::SeqCst)
    }

    pub fn new_server_handle(&self) -> Handle {
        let handle = Handle::new();
        self.server_handles.write().unwrap().push(handle.clone());
        handle
    }
}
