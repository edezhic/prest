//! These docs are focused on technical details. For tutorials check out [prest.blog](https://prest.blog)
#![doc(html_favicon_url = "https://prest.blog/favicon.ico")]

mod db;
mod embed;
mod html;

pub use db::*;
pub use embed::*;
pub use html::*;

#[cfg(not(target_arch = "wasm32"))]
mod host;
#[cfg(not(target_arch = "wasm32"))]
pub use host::*;

#[cfg(target_arch = "wasm32")]
mod service_worker;
#[cfg(target_arch = "wasm32")]
pub use service_worker::*;

// for macro-generated code
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
    stream::{StreamExt, TryStreamExt},
};
pub use once_cell::sync::Lazy;
pub use serde_json::json;
pub use std::{env, sync::Arc};
pub use tower::{self, BoxError, Layer, Service, ServiceBuilder};
pub use tracing::{debug, error, info, trace, warn};

// --- GENERAL UTILS ---

/// Default doctype for HTML
pub const DOCTYPE: PreEscaped<&'static str> = PreEscaped("<!DOCTYPE html>");
/// Default javascript code that registers a service worker from `/sw.js`
pub const REGISTER_SW_SNIPPET: &str =
    "if ('serviceWorker' in navigator) navigator.serviceWorker.register('sw.js', {type: 'module'});";

embed_as!(DefaultAssets from "assets");

/// A little helper to init router and route in a single call to improve formatting
pub fn route<S: Clone + Send + Sync + 'static>(
    path: &str,
    method_router: method_routing::MethodRouter<S>,
) -> Router<S> {
    Router::<S>::new().route(path, method_router)
}

/// Explicit Uuid generation fn
pub fn generate_uuid() -> Uuid {
    Uuid::new_v4()
}

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