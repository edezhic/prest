use crate::*;
use sysinfo::{
    CpuRefreshKind, Disks, Pid, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System,
};

/// Describes collected stats for system resources
#[derive(Debug, Storage, Serialize, Deserialize)]
pub(crate) struct SystemStat {
    pub timestamp: NaiveDateTime,
    pub app_cpu: f32,
    pub other_cpu: f32,
    pub app_ram: u32,
    pub other_ram: u32,
}

state!(SYSTEM_INFO: SystemInfo = async { SystemInfo::init().await });

pub struct SystemInfo {
    pub system: RwLock<System>,
    pub app_pid: Pid,
    pub cores: usize,
    pub ram: u64,
    pub used_disk: RwLock<u32>,
    pub total_disk: u32,
}

impl SystemInfo {
    pub async fn init() -> Self {
        let mut sys =
            System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::everything()));
        sys.refresh_memory();

        let disks = Disks::new_with_refreshed_list();
        let disk = disks
            .list()
            .first()
            .expect("System must have at least one disk");
        let total_disk = disk.total_space().div_ceil(1_000_000) as u32;
        let used_disk = total_disk - disk.available_space().div_ceil(1_000_000) as u32;
        let used_disk = RwLock::new(used_disk);

        let ram = sys.total_memory().div_ceil(1_000_000);

        let host = SystemInfo {
            app_pid: Pid::from_u32(std::process::id()),
            cores: sys.cpus().len(),
            system: RwLock::new(sys),
            ram,
            used_disk,
            total_disk,
        };
        RT.every(1)
            .second()
            .spawn(|| async { SYSTEM_INFO.record().await });

        // Cleanup job: Delete SystemStat entries older than 1 day every 10 minutes
        RT.every(30)
            .minutes()
            .schedule("SystemStat Cleanup", || async {
                match SystemInfo::cleanup_old_stats().await {
                    Ok(deleted_count) => {
                        debug!(target: "gc", "Deleted {} old SystemStat entries", deleted_count);
                    }
                    Err(e) => {
                        warn!(target: "gc", "Failed to cleanup old SystemStat entries: {}", e);
                    }
                }
            });

        // Cleanup job: Delete trace log files older than 1 month every 24 hours
        RT.every(24).hours().schedule("Log Cleanup", || async {
            match LOGS.cleanup_old_traces() {
                Ok(deleted_count) => {
                    debug!(target: "gc", "Deleted {} old trace log files", deleted_count);
                }
                Err(e) => {
                    warn!(target: "gc", "Failed to cleanup old trace log files: {}", e);
                }
            }
        });

        host
    }
    pub async fn record(&self) -> Result {
        self.refresh().await;
        let sys = SYSTEM_INFO.system.read().await;

        let disks = Disks::new_with_refreshed_list();
        let Some(disk) = disks.list().first() else {
            return Err(e!("Disk not found"));
        };
        *SYSTEM_INFO.used_disk.write().await =
            SYSTEM_INFO.total_disk - disk.available_space().div_ceil(1_000_000) as u32;

        let Some(current) = sys.process(SYSTEM_INFO.app_pid) else {
            error!(target: "system info", "Current process not found");
            return Err(e!("Current process not found"));
        };

        let app_ram = current.memory().div_ceil(1_048_576) as u32;
        let used_ram = sys.used_memory().div_ceil(1_048_576) as u32;
        let other_ram = used_ram - app_ram;

        let app_cpu = current.cpu_usage() / SYSTEM_INFO.cores as f32;
        let other_cpu = sys.global_cpu_usage() - app_cpu;

        let stats = SystemStat {
            timestamp: Utc::now().naive_utc(),
            app_cpu,
            other_cpu,
            app_ram,
            other_ram,
        };

        if let Err(e) = stats.save().await {
            warn!(target: "system info", "Failed to save system stats: {e}");
        }
        Ok(())
    }

    pub async fn refresh(&self) {
        let mut sys = self.system.write().await;
        sys.refresh_cpu_all();
        sys.refresh_memory();
        sys.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[self.app_pid]),
            true,
            ProcessRefreshKind::everything(),
        );
    }

    /// Cleanup SystemStat entries older than 1 day
    pub async fn cleanup_old_stats() -> Result<usize> {
        let cutoff_time = Utc::now().naive_utc() - chrono::Duration::days(1);

        // Use SQL DELETE to efficiently remove old entries
        let delete_sql = format!(
            "DELETE FROM SystemStat WHERE timestamp < '{}'",
            cutoff_time.format("%Y-%m-%d %H:%M:%S")
        );

        match DB.write_sql(&delete_sql).await? {
            Payload::Affected(count) => {
                debug!(target: "gc", "Successfully deleted {} SystemStat entries older than {}", count, cutoff_time);
                Ok(count)
            }
            other => {
                warn!(target: "gc", "Unexpected response from cleanup: {:?}", other);
                Ok(0)
            }
        }
    }
}
