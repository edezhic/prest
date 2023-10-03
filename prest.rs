#![allow(dead_code, unused_imports)]

pub use anyhow::{self, Error, Result, bail};
pub use http::{self, header, HeaderMap, HeaderValue, StatusCode};
pub use axum::{
    self,
    body::{Body, HttpBody},
    extract::*,
    response::{IntoResponse, Redirect, Response},
    routing::{any, delete, get, patch, post, put},
    Router,
};
mod middleware;
pub use middleware::*;
mod utils;
pub use utils::*;

#[cfg(feature = "build")]
pub mod build;

pub static REGISTER_SW_SNIPPET: &str = 
    "if ('serviceWorker' in navigator) navigator.serviceWorker.register('sw.js', {type: 'module'});";
