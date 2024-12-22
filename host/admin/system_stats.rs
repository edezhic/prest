use crate::*;
use chrono::{TimeDelta, Utc};
use host::{system_info::SystemStat, LOGS_INFO_NAME, LOGS_TRACES_NAME};

pub(crate) async fn full() -> Result<Markup> {
    const MAX_COUNT: usize = 100;
    let max = Utc::now().naive_utc();
    let min = max - TimeDelta::try_hours(24).unwrap();
    let records = SystemStat::find_in_range_timestamp(&min, &max)?;

    let count = records.len();

    let records = if count > MAX_COUNT {
        records
            .chunks(count.div_ceil(MAX_COUNT))
            .fold(vec![], |mut chunked, stats| {
                let chunk_size = stats.len();
                let chunk = stats.iter().fold(
                    (0.0, 0.0, 0, 0),
                    |(app_cpu, other_cpu, app_ram, other_ram), stat| {
                        (
                            app_cpu + stat.app_cpu,
                            other_cpu + stat.other_cpu,
                            app_ram + stat.app_ram,
                            other_ram + stat.other_ram,
                        )
                    },
                );
                chunked.push(SystemStat {
                    timestamp: stats[0].timestamp,
                    app_cpu: chunk.0 / chunk_size as f32,
                    other_cpu: chunk.1 / chunk_size as f32,
                    app_ram: chunk.2 / chunk_size as u32,
                    other_ram: chunk.3 / chunk_size as u32,
                });
                chunked
            })
    } else {
        records
    };

    let max_cpu = records
        .iter()
        .fold(1.0, |max, r| (r.app_cpu + r.other_cpu).max(max));

    let width = 100.0 / records.len() as f64;

    let mut max_ram: f32 = 0.0;
    let (ram, cpu): (Vec<_>, Vec<_>) = records
        .into_iter()
        .map(|r| {
            let app_ram_prct = r.app_ram as f32 / SYSTEM_INFO.ram as f32 * 100.0;
            let other_ram_prct = r.other_ram as f32 / SYSTEM_INFO.ram as f32 * 100.0;
            let app_cpu = r.app_cpu * (100.0 / max_cpu);
            let other_cpu = r.other_cpu * (100.0 / max_cpu);

            max_ram = max_ram.max(app_ram_prct + other_ram_prct);

            let ram_bar = html!(div $"flex flex-col-reverse" style=(format!("width: {width}%")) {
                div style=(format!("height: {other_ram_prct}%; background-color: rgba(255,255,255,0.3);")) {}
                div style=(format!("height: {app_ram_prct}%; background-color: #22c55e;")) {}
            });
            let cpu_bar = html!(div $"flex flex-col-reverse" style=(format!("width: {width}%")) {
                div style=(format!("height: {other_cpu}%; background-color: rgba(255,255,255,0.3);")) {}
                div style=(format!("height: {app_cpu}%; background-color: #22c55e;")) {}
            });

            (ram_bar, cpu_bar)
        })
        .unzip();

    let total_ram = SYSTEM_INFO.ram.div_ceil(100) as f64 / 10.0;
    let total_ram = format!("{total_ram:.1} GB");
    let max_ram = format!("{max_ram:.1}%");
    let max_cpu = format!("{:.1}", max_cpu);

    ok(html!(
        $"w-full" get="/admin/system_stats" trigger="load delay:30s" swap-this {
            $"w-full flex gap-4" {
                $"w-1/2 h-full flex flex-col" {
                    $"font-bold text-lg" {"CPU usage"}
                    $"text-xs lg:text-sm leading-tight lg:leading-snug flex flex-col lg:flex-row" {
                        span{"Cores: "(SYSTEM_INFO.cores)}
                        span $"hidden lg:block mr-1"{","}
                        span{"max used: "(max_cpu)"%"}
                    }
                    $"h-24 flex border border-neutral-700" {(cpu)}
                }
                $"w-1/2 h-full flex flex-col" {
                    $"font-bold text-lg" {"RAM usage"}
                    $"text-xs lg:text-sm leading-tight lg:leading-snug flex flex-col lg:flex-row" {
                        span{"Total: "(total_ram)}
                        span $"hidden lg:block mr-1"{","}
                        span{"max used: "(max_ram)}
                    }
                    $"h-24 flex border border-neutral-700" {(ram)}
                }
            }
            $"w-full h-6" {}
            (disk_stats().await?)
        }
    ))
}

async fn disk_stats() -> Result<Markup> {
    let used_disk = SYSTEM_INFO.used_disk.read().await;
    let total_disk = SYSTEM_INFO.total_disk;

    let data_dir = &APP_CONFIG.check().data_dir;

    use std::{fs, io, path::PathBuf};

    fn dir_size(path: impl Into<PathBuf>) -> io::Result<u64> {
        fn dir_size(mut dir: fs::ReadDir) -> io::Result<u64> {
            dir.try_fold(0, |acc, file| {
                let file = file?;
                let size = match file.metadata()? {
                    data if data.is_dir() => dir_size(fs::read_dir(file.path())?)?,
                    data => data.len(),
                };
                Ok(acc + size)
            })
        }

        dir_size(fs::read_dir(path.into())?)
    }

    let mut info_path = data_dir.clone();
    info_path.push(LOGS_INFO_NAME);
    let info_size = std::fs::metadata(info_path)?.len();

    let mut traces_path = data_dir.clone();
    traces_path.push(LOGS_TRACES_NAME);
    let traces_size = dir_size(traces_path)?;

    let logs_size = (info_size + traces_size) as f64 / 1_000_000.0;
    let logs_size = format!("{logs_size:.1} MB");

    let mut db_path = data_dir.clone();
    db_path.push(DB_DIRECTORY_NAME);
    let db_size = dir_size(db_path)?;
    let db_size = format!("{:.1} MB", db_size as f64 / 1_000_000.0);

    let used_disk = *used_disk as f64 / total_disk as f64 * 100.0;
    let used_disk = format!("{used_disk:.1}%");
    let total_disk = format!("{:.1}", total_disk as f64 / 1000.0);
    ok(html!(
        $"w-full flex items-center text-[0.6rem] md:text-sm lg:text-base" {
            $"hidden md:block font-bold" {"Disk usage: "} $"block md:hidden font-bold" {"Disk: "}
            $"w-1"{} div {(used_disk)" of "(total_disk)" GB (""DB: "(db_size)", logs: "(logs_size)")"}
        }
        $"hidden lg:block text-xs" {"App data path: " code $"bg-stone-900 p-1" {(data_dir.display())}}
    ))
}
