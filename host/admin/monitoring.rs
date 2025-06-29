use crate::*;
use chrono::{TimeDelta, Utc};
use host::{monitoring::SystemStat, LOGS_INFO_NAME, LOGS_TRACES_NAME};

#[derive(Serialize)]
struct MonitoringData {
    records: Vec<SystemStat>,
    max_ram: u64,
    cores_num: usize,
}

async fn get_stats_for_period(period: &str) -> Result<Vec<SystemStat>> {
    let max = Utc::now().naive_utc();
    let min = match period {
        "5m" => max - TimeDelta::try_minutes(5).unwrap(),
        "15m" => max - TimeDelta::try_minutes(15).unwrap(),
        "30m" => max - TimeDelta::try_minutes(30).unwrap(),
        "1h" => max - TimeDelta::try_hours(1).unwrap(),
        "2h" => max - TimeDelta::try_hours(2).unwrap(),
        "6h" => max - TimeDelta::try_hours(6).unwrap(),
        "12h" => max - TimeDelta::try_hours(12).unwrap(),
        "24h" => max - TimeDelta::try_hours(24).unwrap(),
        _ => max - TimeDelta::try_minutes(15).unwrap(), // Default to 15 minutes
    };
    SystemStat::get_in_timestamp_range(min, max).await
}

pub(crate) async fn data(req: Request) -> Result<impl IntoResponse> {
    let query = req.uri().query().unwrap_or("");
    let period = if query.starts_with("period=") {
        &query[7..] // Skip "period="
    } else {
        "15m"
    };

    Ok(Json(MonitoringData {
        records: get_stats_for_period(period).await?,
        max_ram: SYSTEM_INFO.ram,
        cores_num: SYSTEM_INFO.cores,
    }))
}

pub(crate) async fn container() -> Result<Markup> {
    ok(html!(
        script type="module" {"
            import { loadStats } from './stats.js';
            window.loadStats = loadStats;
        "}

        div $"mb-4 flex items-center gap-3" {
            span $"font-semibold text-sm" { "Time Range:" }
            select
                id="time-range-selector"
                $"bg-stone-900 accent-stone-600 px-2 py-1 text-sm rounded"
                _="on change call loadStats(event.target.value)"
            {
                option value="5m" { "Last 5 minutes" }
                option value="15m" selected { "Last 15 minutes" }
                option value="30m" { "Last 30 minutes" }
                option value="1h" { "Last 1 hour" }
                option value="2h" { "Last 2 hours" }
                option value="6h" { "Last 6 hours" }
                option value="12h" { "Last 12 hours" }
                option value="24h" { "Last 24 hours" }
            }
        }

        a _="on load call loadStats('15m') then remove me" {}
        $"h-[300px]" { canvas #"stats-chart" {} }

        (disk_stats().await?)
    ))
}

async fn disk_stats() -> Result<Markup> {
    let used_disk = SYSTEM_INFO.used_disk.read().await;
    let total_disk = SYSTEM_INFO.total_disk;

    let data_dir = &APP_CONFIG.data_dir;

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
        $"w-full items-center text-[0.6rem] md:text-sm lg:text-base" {
            $"hidden md:block font-bold" {"Disk: "} $"block md:hidden font-bold" {"Disk: "}
            div $"text-xs" {(used_disk)" of "(total_disk)" GB (""DB: "(db_size)", logs: "(logs_size)", in " code $"bg-stone-900 p-1" {(data_dir.display())} ")"}
        }
    ))
}
