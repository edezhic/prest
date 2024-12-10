use crate::*;
use sysinfo::{
    CpuRefreshKind, Disks, Pid, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System,
};

/// Describes collected stats for system resources
#[derive(Debug, Table, Serialize, Deserialize)]
pub struct SystemStat {
    pub timestamp: NaiveDateTime,
    pub app_cpu: f32,
    pub other_cpu: f32,
    pub app_ram: u32,
    pub other_ram: u32,
}

state!(SYSTEM_INFO: SystemInfo = { SystemInfo::init() });

pub struct SystemInfo {
    pub system: RwLock<System>,
    pub app_pid: Pid,
    pub cores: usize,
    pub ram: u64,
    pub used_disk: RwLock<u32>,
    pub total_disk: u32,
}

impl SystemInfo {
    pub fn init() -> Self {
        SystemStat::migrate();
        let mut sys =
            System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::everything()));
        sys.refresh_memory();

        let disks = Disks::new_with_refreshed_list();
        let disk = disks.list().first().unwrap();
        let total_disk = disk.total_space().div_ceil(1_000_000) as u32;
        let used_disk = total_disk - disk.available_space().div_ceil(1_000_000) as u32;
        let used_disk = RwLock::new(used_disk);

        let ram = sys.total_memory().div_ceil(1_048_576);

        let host = SystemInfo {
            app_pid: Pid::from_u32(std::process::id()),
            cores: sys.cpus().len(),
            system: RwLock::new(sys),
            ram,
            used_disk,
            total_disk,
        };
        RT.every(60)
            .seconds()
            .spawn(|| async { SYSTEM_INFO.record().await });
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

        if let Err(e) = stats.save() {
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
}
