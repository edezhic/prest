//! These docs are focused on technical details. For examples check out [prest.blog](https://prest.blog)
#![doc(html_favicon_url = "https://prest.blog/favicon.ico")]
#![allow(dead_code, unused_imports)]

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
use axum::routing::method_routing;
pub use axum::{
    self,
    body::{Body, HttpBody},
    error_handling::{HandleError, HandleErrorLayer},
    extract::{
        self, Extension, Form, FromRequest, FromRequestParts, Host, MatchedPath, NestedPath,
        OriginalUri, Path, Query, Request,
    },
    http::{self, header, HeaderMap, HeaderValue, StatusCode, Uri, Method},
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
use std::ops::Deref;
pub use std::{env, sync::Arc};
pub use tower::{self, BoxError, Layer, Service, ServiceBuilder};
pub use tracing::{trace, debug, info, warn, error};
pub use async_trait::async_trait;

// --- GENERAL UTILS ---

/// Default doctype for HTML
pub const DOCTYPE: PreEscaped<&'static str> = PreEscaped("<!DOCTYPE html>");
/// Default javascript code that registers a service worker from `/sw.js`
pub const REGISTER_SW_SNIPPET: &str =
    "if ('serviceWorker' in navigator) navigator.serviceWorker.register('sw.js', {type: 'module'});";

embed_as!(DefaultAssets from "assets" only "*.css");

/// A little helper to init router and route in a single call to improve formatting
pub fn route<S: Clone + Send + Sync + 'static>(
    path: &str,
    method_router: method_routing::MethodRouter<S>,
) -> Router<S> {
    Router::<S>::new().route(path, method_router)
}

pub fn generate_uuid() -> Uuid {
    Uuid::new_v4()
}
