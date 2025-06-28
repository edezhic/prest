//! These docs are focused on technical details. For tutorials check out [prest.blog](https://prest.blog)
#![doc(html_favicon_url = "https://prest.blog/favicon.ico")]
#![allow(warnings)]

// for macro-generated code inside prest itself
pub(crate) use crate as prest;

#[derive(Debug, Clone, Copy)]
pub struct Prest;

pub use prest_init_macro::init;

#[doc(hidden)]
pub use serde;
pub use serde_derive::{Deserialize, Serialize};

pub use std::future::Future;

// pub use anyhow::{anyhow, bail, Result as AnyhowResult};
pub use async_trait::async_trait;
pub use axum::{
    self,
    body::{Body, HttpBody},
    error_handling::{HandleError, HandleErrorLayer},
    extract::{
        self, Extension, FromRequest, FromRequestParts, Host, MatchedPath, NestedPath, OriginalUri,
        Path, Request,
    },
    http::{self, header, HeaderMap, HeaderValue, Method, StatusCode, Uri},
    middleware::{from_fn, Next},
    response::{ErrorResponse, Html, IntoResponse, Json, Redirect, Response},
    routing::{any, delete, get, patch, post, put},
    Router,
};
// TODO: either do smth with it or get rid of
// pub use axum_htmx::{
//     HxBoosted, HxCurrentUrl, HxEvent, HxHistoryRestoreRequest, HxLocation, HxPrompt, HxPushUrl,
//     HxRedirect, HxRefresh, HxReplaceUrl, HxRequest, HxReselect, HxResponseTrigger, HxReswap,
//     HxRetarget, HxTarget, HxTrigger, HxTriggerName, SwapOption,
// };

pub use bitcode::{deserialize as from_bitcode, serialize as into_bitcode};
pub use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Utc};
pub use futures::{
    future::{join_all, FutureExt},
    stream::{self, Stream, StreamExt, TryStreamExt},
};
pub use serde_json::{
    from_slice as from_json_slice, from_str as from_json_str, json, to_string as to_json_string,
    to_vec as to_json_vec,
};
pub use std::sync::LazyLock as Lazy;
pub use std::{env::var as env_var, sync::Arc};
// pub use tower::{self, BoxError, Layer, Service, ServiceBuilder};
pub use hex;
pub use tracing::{debug, error, info, trace, warn};
pub use uuid::Uuid;

#[doc(hidden)]
pub mod config;
pub use config::APP_CONFIG;

mod result;
#[doc(hidden)]
pub use result::_Somehow;
pub use result::{ok, AnyError, Error, Result, Somehow, OK};

mod vals;
pub use vals::Vals;

#[cfg(feature = "db")]
mod db;
#[cfg(feature = "db")]
pub use db::*;
#[cfg(feature = "embed")]
#[doc(hidden)]
pub mod embed;
#[cfg(feature = "embed")]
pub use embed::Embed;
#[cfg(feature = "embed")]
#[doc(hidden)]
pub use embed::{EmbedRoutes, EmbeddedStruct};
#[cfg(feature = "html")]
mod html;
#[cfg(feature = "html")]
pub use html::*;
#[cfg(feature = "html")]

/// Default doctype for HTML
pub const DOCTYPE: PreEscaped<&'static str> = PreEscaped("<!DOCTYPE html>");
/// Default favicon
pub(crate) static FAVICON: &[u8] = include_bytes!("ui/favicon.ico");

#[cfg(host)]
mod host;
#[cfg(host)]
pub use host::*;

#[cfg(sw)]
mod service_worker;
#[cfg(sw)]
pub use service_worker::*;

// --- GENERAL UTILS ---

/// A little helper to init router and route in a single call to improve formatting
pub fn route<S: Clone + Send + Sync + 'static>(
    path: &str,
    method_router: axum::routing::method_routing::MethodRouter<S>,
) -> Router<S> {
    Router::<S>::new().route(path, method_router)
}

/// Default javascript code that registers a service worker from `/sw.js`
pub const REGISTER_SW_SNIPPET: &str =
    "if ('serviceWorker' in navigator) navigator.serviceWorker.register('sw.js', {type: 'module'});";
/// Returns whether PWA will be built with current configs
pub fn is_pwa() -> bool {
    #[cfg(sw)]
    return true;
    #[cfg(host)]
    match cfg!(debug) {
        true => std::env::var("PWA").map_or(false, |v| v == "debug"),
        false => std::env::var("PWA").map_or(true, |v| v == "release"),
    }
}

/// Shorthand for `PreEscaped(include_str!(...))``
#[macro_export]
macro_rules! include_html {
    ($path: tt) => {
        PreEscaped(include_str!($path))
    };
}
