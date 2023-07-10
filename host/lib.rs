#![feature(let_chains)]

#[macro_use]
extern crate lazy_static;

mod assets;
pub mod auth;
mod config;
pub use config::*;
mod storage;
pub use storage::Storage;

//use axum::response::IntoResponse;
use axum::{routing::get, Router};

pub fn service() -> Router {
    let (auth_svc, session, authn) = auth::init();
    Router::new()
        .route("/authorized", get(|| async {"Authorized!"}))
        .route_layer(auth::RequireAuthzLayer::login())
        .merge(ui::service()) // pure rendering routes
        .merge(auth_svc) // authentication handlers
        .fallback(assets::static_handler)
        .layer(
            tower::ServiceBuilder::new()
                // this compression layer works painfully slow for the sw.wasm module, requires investigation
                //.layer(tower_http::compression::CompressionLayer::new())
                .layer(tower_http::catch_panic::CatchPanicLayer::new())
                .layer(tower_http::trace::TraceLayer::new_for_http())
                .layer(session)
                .layer(authn),
        )
        .route("/health", get(|| async { http::StatusCode::OK }))
}
