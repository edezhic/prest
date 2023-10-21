#![allow(dead_code, unused_imports)]

pub(crate) use crate as prest;

pub use anyhow::{self, Error, Result, bail};
pub use http::{self, Uri, header, HeaderMap, HeaderValue, StatusCode};
pub use axum::{
    self,
    body::{Body, HttpBody},
    Form,
    extract::*,
    response::*,
    routing::{any, delete, get, patch, post, put},
    Router,
    middleware::*,
};
pub use tower::{Layer, Service};
pub use tower_http::*;

pub use once_cell::sync::Lazy;

mod html;
pub use html::*;
pub use html_macro::html;

mod utils;
pub use utils::*;

mod embed;
pub use embed::*;
pub use embed_macro::*;
pub use embed_utils::*;

#[cfg(feature = "build")]
mod build;
#[cfg(feature = "build")]
pub use build::*;

pub const DOCTYPE: PreEscaped<&'static str> = PreEscaped("<!DOCTYPE html>");
pub const REGISTER_SW_SNIPPET: &str = 
    "if ('serviceWorker' in navigator) navigator.serviceWorker.register('sw.js', {type: 'module'});";

use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    trace::TraceLayer,
};
pub fn http_tracing() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
}
