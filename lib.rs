#![allow(dead_code, unused_imports)]
mod html;
mod embed;

pub(crate) use crate as prest;

pub use anyhow::{self, Error, Result, bail, anyhow as anyway};
pub use axum::{
    self,
    body::{Body, HttpBody},
    Form,
    extract::*,
    response::*,
    routing::{any, delete, get, patch, post, put},
    Router,
    middleware::{from_fn, from_fn_with_state, from_extractor, from_extractor_with_state}
};
pub use embed::*;
pub use embed_macro::Embed;
pub use embed_utils::*;
pub use html::*;
pub use html_macro::html;
pub use http ::{self, Uri, header, HeaderMap, HeaderValue, StatusCode};
pub use tower::{self, Layer, Service};
pub use once_cell::sync::Lazy;

/// Default doctype for HTML
pub const DOCTYPE: PreEscaped<&'static str> = PreEscaped("<!DOCTYPE html>");
/// Default javascript code that registers a service worker from `/sw.js`
pub const REGISTER_SW_SNIPPET: &str = 
    "if ('serviceWorker' in navigator) navigator.serviceWorker.register('sw.js', {type: 'module'});";

/// Utility for composition of paths to build artifacts
pub fn out_path(filename: &str) -> String {
    let dir = std::env::var("OUT_DIR").unwrap();
    format!("{dir}/{filename}")
}

#[cfg(feature = "build-pwa")]
mod build_pwa;
#[cfg(feature = "build-pwa")]
pub use build_pwa::*;

use std::net::SocketAddr;

/// Configuration for the server
pub struct ServeOptions {
    pub addr: SocketAddr,
}
impl Default for ServeOptions {
    fn default() -> Self {
        Self {
            addr: SocketAddr::from(([0, 0, 0, 0], 80))
        }
    }
}

#[cfg(feature = "host")]
pub async fn serve(router: Router, opts: ServeOptions) {
    let svc = router.into_make_service();
    hyper_server::bind(opts.addr).serve(svc).await.unwrap();
}

#[cfg(feature = "sw")]
mod sw;
#[cfg(feature = "sw")]
pub use sw::*;

#[cfg(all(target = "wasm32-wasi", feature = "host-wasi"))]
pub async fn serve(router: Router, opts: ServeOptions) { 
    use hyper::server::conn::Http;
    use tokio::net::TcpListener;       
    let listener = TcpListener::bind(opts.addr).await.unwrap();
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let svc = router.clone();
        tokio::task::spawn(async move {
            if let Err(err) = Http::new().serve_connection(stream, svc).await {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}    

/// A CSS response.
///
/// Will automatically get `Content-Type: text/css`.
#[derive(Clone, Copy, Debug)]
#[must_use]
pub struct Css<T>(pub T);
impl<T> IntoResponse for Css<T>
where
    T: Into<Body>,
{
    fn into_response(self) -> Response {
        (
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/css"),
            )],
            self.0.into(),
        )
            .into_response()
    }
}
impl<T> From<T> for Css<T> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

/// A favicon response.
///
/// Will automatically get `Content-Type: image/x-icon`.
#[derive(Clone, Copy, Debug)]
#[must_use]
pub struct Favicon<T>(pub T);
impl<T> IntoResponse for Favicon<T>
where
    T: Into<Body>,
{
    fn into_response(self) -> Response {
        (
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static("image/x-icon"),
            )],
            self.0.into(),
        )
            .into_response()
    }
}
impl<T> From<T> for Favicon<T> {
    fn from(inner: T) -> Self {
        Self(inner)
    }
}

use std::{path::PathBuf, ffi::OsStr};
/// Utility that attempts to find the path of the current build's target path
pub fn find_target_dir() -> Option<String> {
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