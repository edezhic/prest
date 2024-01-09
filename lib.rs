//! These docs are focused on technical details. For tutorials check out [prest.blog](https://prest.blog)
#![doc(html_favicon_url = "https://prest.blog/favicon.ico")]

// for macro-generated code inside prest itself
pub(crate) use crate as prest;

pub use anyhow::{anyhow, bail, Error, Result};
pub use async_trait::async_trait;
use axum::routing::method_routing;
pub use axum::{
    self,
    body::{Body, HttpBody},
    error_handling::{HandleError, HandleErrorLayer},
    extract::{
        self, Extension, Form, FromRequest, FromRequestParts, Host, MatchedPath, NestedPath,
        OriginalUri, Path, Query, Request,
    },
    http::{self, header, HeaderMap, HeaderValue, Method, StatusCode, Uri},
    middleware::{from_extractor, from_extractor_with_state, from_fn, from_fn_with_state, Next},
    response::*,
    routing::{any, delete, get, patch, post, put},
    Router,
};
pub use futures::{
    executor::block_on,
    stream::{self, Stream, StreamExt, TryStreamExt},
};
pub use once_cell::sync::Lazy;
pub use serde_json::json;
pub use std::{env, sync::Arc, convert::Infallible};
pub use tower::{self, BoxError, Layer, Service, ServiceBuilder};
pub use tracing::{debug, error, info, trace, warn};
pub use uuid::Uuid;

#[cfg(feature = "db")]
mod db;
#[cfg(feature = "db")]
pub use db::*;
#[cfg(feature = "embed")]
mod embed;
#[cfg(feature = "embed")]
pub use embed::*;
#[cfg(feature = "embed")]
embed_as!(DefaultAssets from "assets");

#[cfg(feature = "html")]
mod html;
#[cfg(feature = "html")]
pub use html::*;
#[cfg(feature = "html")]
/// Default doctype for HTML
pub const DOCTYPE: PreEscaped<&'static str> = PreEscaped("<!DOCTYPE html>");

#[cfg(not(target_arch = "wasm32"))]
mod host;
#[cfg(not(target_arch = "wasm32"))]
pub use host::*;

#[cfg(target_arch = "wasm32")]
mod service_worker;
#[cfg(target_arch = "wasm32")]
pub use service_worker::*;

// --- GENERAL UTILS ---

/// A little helper to init router and route in a single call to improve formatting
pub fn route<S: Clone + Send + Sync + 'static>(
    path: &str,
    method_router: method_routing::MethodRouter<S>,
) -> Router<S> {
    Router::<S>::new().route(path, method_router)
}

/// Default javascript code that registers a service worker from `/sw.js`
pub const REGISTER_SW_SNIPPET: &str =
    "if ('serviceWorker' in navigator) navigator.serviceWorker.register('sw.js', {type: 'module'});";
pub fn is_pwa() -> bool {
    #[cfg(target_arch = "wasm32")]
    return true;
    #[cfg(not(target_arch = "wasm32"))]
    {
        #[cfg(debug_assertions)]
        return std::env::var("PWA").map_or(false, |v| v == "debug");
        #[cfg(not(debug_assertions))]
        return std::env::var("PWA").map_or(true, |v| v == "release");
    }
}
