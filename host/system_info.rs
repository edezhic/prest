use crate::*;

use sysinfo::{CpuRefreshKind, Pid, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};

state!(SYSTEM_INFO: SystemInfo = async {
    let sys = System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::everything()));
    let host = SystemInfo {
        app_pid: Pid::from_u32(std::process::id()),
        cores: sys.cpus().len(),
        system: RwLock::new(sys),
    };
    host.refresh().await;
    RT.every(300).milliseconds().spawn(|| async { SYSTEM_INFO.refresh().await });
    host
});

pub struct SystemInfo {
    pub system: RwLock<System>,
    pub app_pid: Pid,
    pub cores: usize,
}

impl SystemInfo {
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
