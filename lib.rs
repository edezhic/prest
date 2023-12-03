//! These docs are focused on technical details. For examples check out [prest.blog](https://prest.blog)
#![doc(html_favicon_url = "https://prest.blog/favicon.ico")]
#![allow(dead_code, unused_imports)]

mod html;
mod embed;

pub(crate) use crate as prest;

pub use anyhow::{self, Error, Result, bail, anyhow as anyway};
pub use axum::{
    self,
    body::{Body, HttpBody},
    extract::{self, Extension, Form, FromRequest, FromRequestParts, Host, MatchedPath, NestedPath, OriginalUri, Path, Query, Request, State},
    response::*,
    routing::{any, delete, get, patch, post, put},
    Router,
    middleware::{Next, from_fn, from_fn_with_state, from_extractor, from_extractor_with_state},
    http::{self, Uri, header, HeaderMap, HeaderValue, StatusCode}
};
pub use embed::*;
pub use embed_macro::Embed;
pub use embed_utils::*;
pub use html::*;
pub use html_macro::html;
pub use futures::{executor::block_on, stream::{StreamExt, TryStreamExt}};
pub use tower::{self, Layer, Service};
pub use once_cell::sync::Lazy;
pub use std::{sync::Arc, env};

/// Default doctype for HTML
pub const DOCTYPE: PreEscaped<&'static str> = PreEscaped("<!DOCTYPE html>");
/// Default javascript code that registers a service worker from `/sw.js`
pub const REGISTER_SW_SNIPPET: &str = 
    "if ('serviceWorker' in navigator) navigator.serviceWorker.register('sw.js', {type: 'module'});";
/// TODO
pub const RELEASE: bool = cfg!(not(debug_assertions));

/// Utility for composition of paths to build artifacts
pub fn out_path(filename: &str) -> String {
    let dir = std::env::var("OUT_DIR").unwrap();
    format!("{dir}/{filename}")
}

#[cfg(feature = "build-pwa")]
mod build_pwa;
#[cfg(feature = "build-pwa")]
pub use build_pwa::*;

#[cfg(not(target_arch = "wasm32"))]
mod host {
    use super::*;
    use std::net::SocketAddr;
    use tower::ServiceBuilder;
    use tracing_subscriber::{layer::SubscriberExt, filter::LevelFilter, util::SubscriberInitExt};
    use tokio::{runtime::Runtime, net::TcpListener};
    use tower_http::{catch_panic::CatchPanicLayer, limit::RequestBodyLimitLayer, trace::TraceLayer, compression::CompressionLayer};
    
    /// Configuration for the server
    pub struct ServeOptions {
        pub addr: SocketAddr,
        pub request_body_limit: usize,
        pub log_filter: LevelFilter
    
    }
    impl Default for ServeOptions {
        fn default() -> Self {
            let port = if let Ok(v) = std::env::var("PORT") {
                v.parse::<u16>().unwrap_or(80)
            } else {
                80
            };
            Self {
                addr: SocketAddr::from(([0, 0, 0, 0], port)),
                request_body_limit: 1000000,
                log_filter: LevelFilter::DEBUG
            }
        }
    }
    
    /// Util trait to add serve function to the Router
    pub trait Serving {
        fn serve(self, opts: ServeOptions);
    }
    
    /// Start tokio+hyper based server
    impl Serving for Router {
        fn serve(mut self, opts: ServeOptions) {
            tracing_subscriber::registry()
                .with(opts.log_filter)
                .with(tracing_subscriber::fmt::layer())
                .init();

            let host_services = ServiceBuilder::new()
                .layer(CatchPanicLayer::new())
                .layer(RequestBodyLimitLayer::new(opts.request_body_limit))
                .layer(CompressionLayer::new())
                .layer(TraceLayer::new_for_http());
            
            self = self.layer(host_services);
                
            Runtime::new().unwrap().block_on(async move {
                let listener = TcpListener::bind(opts.addr).await.unwrap();
                axum::serve(listener, self).await
            }).unwrap();
        }
    }
}
#[cfg(not(target_arch = "wasm32"))]
pub use host::*;

#[cfg(target_arch = "wasm32")]
mod sw;
#[cfg(target_arch = "wasm32")]
pub use sw::*;

/// Utility that attempts to find the path of the current build's target path
pub fn find_target_dir() -> Option<String> {
    use std::{path::PathBuf, ffi::OsStr};
    if let Some(target_dir) = std::env::var_os("CARGO_TARGET_DIR") {
        let target_dir = PathBuf::from(target_dir);
        if target_dir.is_absolute() {
            if let Some(str) = target_dir.to_str() {
                return Some(str.to_owned());
            } else {
                return None;
            }
        } else {
            return None;
        };
    }

    let mut dir = PathBuf::from(out_path(""));
    loop {
        if dir.join(".rustc_info.json").exists()
            || dir.join("CACHEDIR.TAG").exists()
            || dir.file_name() == Some(OsStr::new("target"))
                && dir
                    .parent()
                    .map_or(false, |parent| parent.join("Cargo.toml").exists())
        {
            if let Some(str) = dir.to_str() {
                return Some(str.to_owned());
            } else {
                return None;
            }
        }
        if dir.pop() {
            continue;
        }
        return None;
    }
}
