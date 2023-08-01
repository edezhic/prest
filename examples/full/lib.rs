#![feature(let_chains)]

#[macro_use]
extern crate lazy_static;

pub mod ui;

#[cfg(host)]
mod assets;
#[cfg(host)]
pub mod auth;
#[cfg(host)]
pub mod config;
#[cfg(host)]
mod storage;
#[cfg(host)]
pub use storage::Storage;

#[cfg(host)]
pub fn service() -> axum::Router {
    use axum::routing::get;
    let (auth_svc, session, authn) = auth::init();
    axum::Router::new()
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

#[cfg(sw)]
// wasm_bindgen macro generates wasm-js bindings for the fetch event listener
#[wasm_bindgen::prelude::wasm_bindgen]
pub async fn service(host: &str, event: web_sys::FetchEvent) {
    use tower_service::Service;
    pwrs_sw::set_panic_hook();
    let request = pwrs_sw::fetch_into_axum_request(&event).await;
    // process only requests to our host
    if request.uri().host() != Some(host) {
        return;
    }
    // pass the request to the ui service
    let Ok(response) = ui::service().call(request).await else { return };
    // proceed only if OK
    if response.status().as_u16() != 200 {
        return;
    }
    let promise = pwrs_sw::axum_response_into_promise(response);
    event.respond_with(&promise).unwrap();
}
