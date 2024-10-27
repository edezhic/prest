//! These docs are focused on technical details. For tutorials check out [prest.blog](https://prest.blog)
#![doc(html_favicon_url = "https://prest.blog/favicon.ico")]

// for macro-generated code inside prest itself
pub(crate) use crate as prest;

pub use serde;
pub use serde_derive::{Deserialize, Serialize};

pub use anyhow::{anyhow, bail, Result as AnyhowResult};
pub use async_trait::async_trait;
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
    response::{
        AppendHeaders, ErrorResponse, Html, IntoResponse, IntoResponseParts, Json, Redirect,
        Response, ResponseParts,
    },
    routing::{any, delete, get, patch, post, put},
    Router,
};
pub use axum_htmx::{
    HxBoosted, HxCurrentUrl, HxEvent, HxHistoryRestoreRequest, HxLocation, HxPrompt, HxPushUrl,
    HxRedirect, HxRefresh, HxReplaceUrl, HxRequest, HxReselect, HxResponseTrigger, HxReswap,
    HxRetarget, HxTarget, HxTrigger, HxTriggerName, SwapOption,
};
pub use axum_valid::*;
pub use futures::{
    executor::block_on as await_fut,
    stream::{self, Stream, StreamExt, TryStreamExt},
};
pub use serde_json::{
    from_slice as from_json_slice, from_str as from_json_str, json, to_string as to_json_string,
};
pub use std::sync::LazyLock as Lazy;
pub use std::{convert::Infallible, env, sync::Arc};
pub use toml::{Table as TomlTable, Value as TomlValue};
pub use tower::{self, BoxError, Layer, Service, ServiceBuilder};
pub use tracing::{debug, error, info, trace, warn};
pub use uuid::Uuid;

mod config;
pub use config::*;

#[cfg(feature = "db")]
mod db;
#[cfg(feature = "db")]
pub use db::*;
#[cfg(feature = "embed")]
mod embed;
#[cfg(feature = "embed")]
pub use embed::*;
#[cfg(feature = "html")]
mod html;
#[cfg(feature = "html")]
pub use html::*;
#[cfg(feature = "html")]
/// Default doctype for HTML
pub const DOCTYPE: PreEscaped<&'static str> = PreEscaped("<!DOCTYPE html>");
/// Default favicon
pub static FAVICON: &[u8] = include_bytes!("favicon.ico");

#[cfg(host)]
mod host;
#[cfg(host)]
pub use host::*;

#[cfg(sw)]
mod service_worker;
#[cfg(sw)]
pub use service_worker::*;

// --- GENERAL UTILS ---

/// Starting point for prest apps that performs basic setup
#[macro_export]
macro_rules! init {
    ($(tables $($table:ident),+)?) => {
        {
            // let manifest = include_str!("../Cargo.toml");
            let manifest = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));
            #[cfg(not(target_arch = "wasm32"))]
            let ___ = prest::dotenv();
            prest::init_tracing_subscriber();
            let config = APP_CONFIG.init(manifest, env!("CARGO_MANIFEST_DIR"));
            prest::DB.init();
            $(
                $( $table::prepare_table(); )+
            )?
            let _ = *prest::RT;
            prest::info!("Initialized {} v{}", config.name, config.version);
        }
    };
}

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

#[macro_export]
macro_rules! include_html {
    ($path: tt) => {
        PreEscaped(include_str!($path))
    };
}

/// Basic Result alias with [`enum@prest::Error`]`
pub type Result<T = (), E = Error> = std::result::Result<T, E>;

use thiserror::Error;

/// Error type used across prest codebase
#[derive(Error, Debug)]
pub enum Error {
    #[error("Internal")]
    Internal,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Not found")]
    NotFound,
    #[error(transparent)]
    Env(#[from] std::env::VarError),
    #[cfg(all(host, feature = "auth"))]
    #[error(transparent)]
    Session(#[from] tower_sessions::session_store::Error),
    #[cfg(all(host, feature = "auth"))]
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[cfg(all(host, feature = "auth"))]
    #[error(transparent)]
    OAuth(#[from] openidconnect::ClaimsVerificationError),
    #[cfg(feature = "db")]
    #[error(transparent)]
    GlueSQL(#[from] gluesql::core::error::Error),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    FormRejection(#[from] axum::extract::rejection::FormRejection),
    #[cfg(host)]
    #[error(transparent)]
    RuSSH(#[from] russh::Error),
    #[cfg(host)]
    #[error(transparent)]
    RuSFTP(#[from] russh_sftp::client::error::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        error!("{self}");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

pub const OK: Result<(), Error> = Result::Ok(());
pub const fn ok<T: IntoResponse>(resp: T) -> Result<T, Error> {
    Ok(resp)
}
