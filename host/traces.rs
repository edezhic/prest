use crate::*;

use rev_buf_reader::RevBufReader;
use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    trace::TraceLayer,
};
use tracing::{Level, Span};
pub use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::{
    filter::Targets,
    fmt::{self, time::ChronoUtc},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

use std::{
    fs::OpenOptions,
    io::{BufRead, Write},
    path::PathBuf,
};

pub struct Log(PathBuf);
impl Log {
    pub fn write(&self, data: &str) {
        let mut f = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&self.0)
            .expect("Unable to open log for writing");
        f.write_all(data.as_bytes()).expect("Unable to write logs");
    }

    pub fn read_last_lines(&self, count: usize) -> Vec<String> {
        let Ok(file) = OpenOptions::new().read(true).open(&self.0) else {
            return vec![];
        };
        let buf = RevBufReader::new(file);
        buf.lines()
            .take(count)
            .map(|l| l.expect("Could not parse line"))
            .collect()
    }
}

state!(LOG: Log = {
    let AppConfig {
        name, ..
    } = APP_CONFIG.check();

    let project_dirs = prest::ProjectDirs::from("", "", &name).unwrap();
    let mut log_path = project_dirs.data_dir().to_path_buf();
    log_path.push("log");
    Log(log_path)
});

state!(DEBUG_LOG: Log = {
    let AppConfig {
        name, ..
    } = APP_CONFIG.check();

    let project_dirs = prest::ProjectDirs::from("", "", &name).unwrap();
    let mut log_path = project_dirs.data_dir().to_path_buf();
    log_path.push("detailed_log");
    Log(log_path)
});

pub struct Logger;
impl std::io::Write for Logger {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let Ok(log) = std::str::from_utf8(buf) else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Not UTF-8 log",
            ));
        };
        let Ok(log) = ansi_to_html::convert(log) else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Not ANSI log",
            ));
        };
        LOG.write(&log);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::writer::MakeWriter<'a> for Logger {
    type Writer = Logger;

    fn make_writer(&'a self) -> Self::Writer {
        Logger
    }
}

pub struct DebugLogger;
impl std::io::Write for DebugLogger {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let Ok(log) = std::str::from_utf8(buf) else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Not UTF-8 log",
            ));
        };
        let Ok(log) = ansi_to_html::convert(log) else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Not ANSI log",
            ));
        };
        DEBUG_LOG.write(&log);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> tracing_subscriber::fmt::writer::MakeWriter<'a> for DebugLogger {
    type Writer = DebugLogger;

    fn make_writer(&'a self) -> Self::Writer {
        DebugLogger
    }
}

fn pretty_filter() -> EnvFilter {
    EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env()
        .unwrap()
        .add_directive("sqlparser::parser=info".parse().unwrap())
        .add_directive("tower_sessions_core=info".parse().unwrap())
        .add_directive("h2=info".parse().unwrap())
        .add_directive("hyper=info".parse().unwrap())
        .add_directive("rustls=info".parse().unwrap())
        .add_directive("reqwest=info".parse().unwrap())
        .add_directive("russh=info".parse().unwrap())
        .add_directive("sled=info".parse().unwrap())
}

fn debug_filter() -> Targets {
    Targets::new()
        .with_target("sled::tree", Level::INFO)
        .with_target("sled::pagecache", Level::INFO)
        .with_target("sqlparser::parser", Level::INFO)
        .with_target("prest::host::traces", Level::DEBUG) // hide requests to /admin/logs
        .with_default(LevelFilter::TRACE)
}

/// Initializes log collection
pub fn init_tracing_subscriber() {
    let debug_layer = fmt::layer()
        .map_writer(move |_| DebugLogger)
        .with_filter(debug_filter());

    let admin_layer = fmt::layer()
        .with_timer(ChronoUtc::new("%k:%M:%S".to_owned()))
        .map_writer(move |_| Logger)
        .with_filter(pretty_filter());

    let shell_layer = fmt::layer()
        .with_timer(ChronoUtc::new("%k:%M:%S".to_owned()))
        .with_filter(pretty_filter());

    tracing_subscriber::registry()
        .with(debug_layer)
        .with(admin_layer)
        .with(shell_layer)
        .init()
}

pub fn trace_layer() -> TraceLayer<
    SharedClassifier<ServerErrorsAsFailures>,
    impl Fn(&Request<Body>) -> Span + Clone + Copy,
    (),
    impl Fn(&Response<Body>, std::time::Duration, &Span) + Clone + Copy,
    (),
    (),
> {
    let layer = TraceLayer::new_for_http()
        .on_eos(())
        .on_body_chunk(())
        .on_request(())
        .on_response(
            |resp: &Response, latency: std::time::Duration, span: &Span| {
                let millis = latency.as_secs_f64() * 1000.0;
                let status = resp.status();
                if let Some(metadata) = span.metadata() {
                    let mut level = *metadata.level();
                    if super::filter_response(resp) {
                        level = Level::TRACE;
                    }
                    match level {
                        Level::DEBUG => tracing::debug!("'{status}' in {millis:.1}ms"),
                        Level::TRACE => tracing::trace!("'{status}' in {millis:.1}ms"),
                        _ => {}
                    }
                }
            },
        )
        .make_span_with(|request: &Request| {
            let method = request.method().as_str();
            let uri = request.uri();
            let path = uri.path();
            let uri = if path.starts_with("/auth/") && uri.query().is_some() {
                path.to_owned() + "?[redacted]"
            } else {
                uri.to_string()
            };

            if super::filter_request(request) {
                return tracing::trace_span!("->", method, uri);
            }

            match *request.method() {
                Method::GET => tracing::debug_span!("-> GET   ", uri),
                Method::POST => tracing::debug_span!("-> POST  ", uri),
                Method::PUT => tracing::debug_span!("-> PUT   ", uri),
                Method::PATCH => tracing::debug_span!("-> PATCH ", uri),
                Method::DELETE => tracing::debug_span!("-> DELETE", uri),
                _ => tracing::debug_span!("->", method, uri),
            }
        });
    layer
}
