use crate::*;

use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    trace::TraceLayer,
};
use tracing::{Level, Span};
pub use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::{
    fmt::{self, time::ChronoUtc},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

use std::sync::RwLock;

state!(LOG: RwLock<String> = { RwLock::default() });

pub struct Logger;

impl std::io::Write for Logger {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let Ok(log) = std::str::from_utf8(buf) else {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Not UTF-8 log"));
        };
        let Ok(log) = ansi_to_html::convert(log) else {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Not ANSI log"));
        };
        LOG.write().unwrap().push_str(&log);
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

fn make_filter() -> EnvFilter {
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
        .add_directive("pingora_proxy=info".parse().unwrap())
        .add_directive("pingora_core=info".parse().unwrap())
        .add_directive("pingora_pool=info".parse().unwrap())
        .add_directive("pingora_core::server=warn".parse().unwrap())
}

pub fn init_tracing_subscriber() {
    let admin_layer = fmt::layer()
        .with_timer(ChronoUtc::new("%k:%M:%S".to_owned()))
        .map_writer(move |_| Logger)
        .with_filter(make_filter());
    
    let subscriber = tracing_subscriber::registry().with(admin_layer);

    //#[cfg(debug_assertions)]
    let shell_layer = fmt::layer()
        .with_timer(ChronoUtc::new("%k:%M:%S".to_owned()))
        .with_filter(make_filter());
    
    //#[cfg(debug_assertions)]
    let subscriber = subscriber.with(shell_layer);

    subscriber.init()
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
        .on_response(|resp: &Response, latency: std::time::Duration, span: &Span| {
            let millis = latency.as_secs_f64() * 1000.0;
            let status = resp.status();
            if let Some(metadata) = span.metadata() {
                match *metadata.level() {
                    Level::DEBUG => tracing::debug!("'{status}' in {millis:.1}ms"),
                    Level::TRACE => tracing::trace!("'{status}' in {millis:.1}ms"),
                    _ => {}
                }
            }
        })
        .make_span_with(|request: &Request| {
            let method = request.method().as_str();
            let uri = request.uri();
            let path = uri.path();
            let uri = if path.starts_with("/auth/") && uri.query().is_some() {
                path.to_owned() + "?[redacted]"
            } else {
                uri.to_string()
            };

            if super::internal_req(request) {
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

