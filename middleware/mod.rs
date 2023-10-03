#[cfg(feature = "htmx")]
mod htmx;
#[cfg(feature = "htmx")]
pub use htmx::*;
#[cfg(feature = "embed")]
mod embed;
#[cfg(feature = "embed")]
pub use embed::embed;

pub use tower::{Layer, Service};
pub use axum::middleware::*;
pub use tower_http::*;

use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    trace::TraceLayer,
};

pub fn http_tracing() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
}

