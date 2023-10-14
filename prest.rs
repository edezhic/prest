#![allow(dead_code, unused_imports)]

pub use anyhow::{self, Error, Result, bail};
pub use http::{self, header, HeaderMap, HeaderValue, StatusCode};
pub use axum::{
    self,
    body::{Body, HttpBody},
    Form,
    extract::*,
    response::{IntoResponse, Redirect, Response, Html},
    routing::{any, delete, get, patch, post, put},
    Router,
};
pub use html_macro::html;
mod html;
pub use html::*;
mod middleware;
pub use middleware::*;
mod utils;
pub use utils::*;

#[cfg(feature = "build")]
pub mod build;

//pub type SyncState<T> = Lazy<Arc<Mutex<T>>>;

pub static REGISTER_SW_SNIPPET: &str = 
    "if ('serviceWorker' in navigator) navigator.serviceWorker.register('sw.js', {type: 'module'});";

pub const DOCTYPE: PreEscaped<&'static str> = PreEscaped("<!DOCTYPE html>");
