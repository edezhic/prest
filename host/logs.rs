use crate::*;

use rev_buf_reader::RevBufReader;
use std::{
    fs::OpenOptions,
    io::{BufRead, Read, Write},
    path::PathBuf,
};
use tracing::Level;
pub use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::{
    filter::Targets,
    fmt::{self, time::ChronoUtc},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

state!(LOGS: Logs = {
    let AppConfig {
        name, ..
    } = APP_CONFIG.check();

    let project_dirs = prest::ProjectDirs::from("", "", &name).unwrap();
    let mut info_path = project_dirs.data_dir().to_path_buf();
    info_path.push("info");

    let mut traces_path = project_dirs.data_dir().to_path_buf();
    traces_path.push("traces");

    Logs {
        info: Log(info_path),
        traces: Log(traces_path),
    }
});

/// Holds path to the log file
pub struct Log(PathBuf);

/// Holds paths to the log files
pub struct Logs {
    pub info: Log,
    pub traces: Log,
}

impl Logs {
    pub fn write(&self, data: &str, detailed: bool) {
        let path = match detailed {
            true => &self.traces.0,
            false => &self.info.0,
        };

        let mut f = OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .expect("Unable to open log for writing");

        f.write_all(data.as_bytes()).expect("Unable to append log");
        if detailed {
            f.write_all(b",").expect("Unable to append comma to the latest trace");
        }
    }

    pub fn latest_info(&self, count: usize) -> Vec<String> {
        let Ok(file) = OpenOptions::new().read(true).open(&self.info.0) else {
            return vec![];
        };
        let buf = RevBufReader::new(file);
        buf.lines()
            .take(count)
            .map(|l| l.expect("Could not parse line"))
            .collect()
    }

    pub fn traces(&self) -> String {
        let Ok(mut file) = OpenOptions::new().read(true).open(&self.traces.0) else {
            return "Failed to open traces file".into();
        };
        let mut entries = "[".to_owned();
        if let Err(e) = file.read_to_string(&mut entries) {
            return format!("Failed to read traces file: {e}");
        };
        entries.push_str("]");
        entries
    }
}

struct LogWriter {
    detailed: bool,
}
impl std::io::Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let Ok(log) = std::str::from_utf8(buf) else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Not UTF-8 log",
            ));
        };

        if self.detailed {
            LOGS.write(log, self.detailed);
        } else {
            let Ok(log) = ansi_to_html::convert(log) else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Not ANSI log",
                ));
            };
            LOGS.write(&log, self.detailed);
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

struct MakeInfoLogWriter;
impl<'a> tracing_subscriber::fmt::writer::MakeWriter<'a> for MakeInfoLogWriter {
    type Writer = LogWriter;
    fn make_writer(&'a self) -> Self::Writer {
        LogWriter { detailed: false }
    }
}
struct MakeTracesLogWriter;
impl<'a> tracing_subscriber::fmt::writer::MakeWriter<'a> for MakeTracesLogWriter {
    type Writer = LogWriter;
    fn make_writer(&'a self) -> Self::Writer {
        LogWriter { detailed: true }
    }
}

fn info_filter() -> EnvFilter {
    EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env()
        .unwrap()
        .add_directive("sqlparser::parser=info".parse().unwrap())
        .add_directive("sqlparser::dialect=info".parse().unwrap())
        .add_directive("tower_sessions_core=info".parse().unwrap())
        .add_directive("h2=info".parse().unwrap())
        .add_directive("hyper=info".parse().unwrap())
        .add_directive("rustls=info".parse().unwrap())
        .add_directive("reqwest=info".parse().unwrap())
        .add_directive("russh=info".parse().unwrap())
        .add_directive("sled=info".parse().unwrap())
}

fn traces_filter() -> Targets {
    Targets::new()
        .with_target("sled::tree", Level::INFO)
        .with_target("sled::pagecache", Level::INFO)
        .with_target("sqlparser::parser", Level::INFO)
        .with_target("sqlparser::dialect", Level::INFO)
        .with_target("prest::host::traces", Level::DEBUG)
        .with_target("async_io", Level::DEBUG)
        .with_target("polling", Level::DEBUG)
        .with_target("russh", Level::INFO)
        .with_default(LevelFilter::TRACE)
}

/// Initializes log collection and writing into files
pub fn init_tracing_subscriber() {
    let traces_layer = fmt::layer()
        .json()
        .map_writer(move |_| MakeTracesLogWriter)
        .with_filter(traces_filter());

    let info_layer = fmt::layer()
        .with_timer(ChronoUtc::new("%m/%d %k:%M:%S".to_owned()))
        .with_file(false)
        .with_target(false)
        .map_writer(move |_| MakeInfoLogWriter)
        .with_filter(info_filter());

    #[cfg(debug_assertions)]
    let shell_layer = fmt::layer()
        .with_timer(ChronoUtc::new("%k:%M:%S".to_owned()))
        .with_filter(info_filter());

    let registry = tracing_subscriber::registry()
        .with(traces_layer)
        .with(info_layer);

    #[cfg(debug_assertions)]
    let registry = registry.with(shell_layer);

    let _ = *LOGS;

    registry.init()
}
