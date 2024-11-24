use std::sync::atomic::{AtomicBool, Ordering};

use axum_server::Handle;

use crate::*;

state!(SHUTDOWN: Shutdown = { Shutdown::default() });

pub async fn listen_shutdown() {
    match tokio_signal::unix::signal(tokio_signal::unix::SignalKind::terminate()) {
        Ok(mut sigterm) => {
            sigterm.recv().await;
            warn!("Received shutdown(SIGTERM) signal, initiating");
            SHUTDOWN.initiate();
        }
        Err(err) => {
            error!("Error listening for shutdown(SIGTERM) signal: {}", err);
        }
    }
}

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
            self.initiated.store(true, Ordering::SeqCst);
        }
        // stopping the servers
        for handle in self.server_handles.read().unwrap().iter() {
            handle.graceful_shutdown(Some(std::time::Duration::from_secs(1)))
        }
        debug!("Sent graceful shutdown signals for servers");

        // awaiting currently running scheduled tasks
        while self.scheduled_task_running.load(Ordering::SeqCst) {
            continue;
        }
        debug!("Awaited scheduled tasks completion");
        
        // flushing dirty db writes
        #[cfg(feature = "db")]
        DB.flush();
        debug!("Flushed the DB");

        warn!("Finished shutdown procedures");
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
