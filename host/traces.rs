use crate::*;

use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    trace::TraceLayer,
};
use tracing::Span;
pub use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::{
    fmt::{self, time::ChronoUtc},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

pub fn init_tracing_subscriber() {
    let _env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env()
        .unwrap()
        .add_directive("sqlparser::parser=info".parse().unwrap())
        .add_directive("tower_sessions_core=info".parse().unwrap())
        .add_directive("h2=info".parse().unwrap())
        .add_directive("hyper=info".parse().unwrap())
        .add_directive("rustls=info".parse().unwrap())
        .add_directive("reqwest=info".parse().unwrap())
        .add_directive("sled=info".parse().unwrap());
    let fmt_layer = fmt::layer();
    #[cfg(debug_assertions)]
    let fmt_layer = fmt_layer
        .with_timer(ChronoUtc::new("%k:%M:%S%.3f".to_owned()))
        .with_filter(_env_filter);

    let _ = tracing_subscriber::registry().with(fmt_layer).try_init();
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
            |resp: &Response, latency: std::time::Duration, _: &Span| {
                let millis = latency.as_secs_f64() * 1000.0;
                let status = resp.status();
                tracing::debug!("processed with '{status}' in {millis:.2}ms")
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

            if internal_req(request) {
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

const INTERNAL_PATHS: [&str; 2] = ["/tower-livereload", "/default-view-transition"];
fn internal_req(request: &Request) -> bool {
    let path = request.uri().path();
    for internal in INTERNAL_PATHS {
        if path.starts_with(internal) {
            return true;
        }
    }
    false
}
