#![feature(let_chains)]

pub mod ui;

#[derive(rust_embed::RustEmbed, Clone, Copy)]
#[folder = "./pub"]
struct Assets;

#[cfg(host)]
pub fn service() -> axum::Router {
    axum::Router::new()
        .merge(ui::service())
        .layer(pwrs_host::Embed::load(Assets))
        .layer(tower_http::trace::TraceLayer::new_for_http())
}

#[cfg(sw)]
// wasm_bindgen macro generates wasm-js bindings for the fetch event listener
#[wasm_bindgen::prelude::wasm_bindgen]
pub async fn service(host: &str, event: web_sys::FetchEvent) {
    pwrs_sw::set_panic_hook();
    let request = pwrs_sw::fetch_into_axum_request(&event).await;
    // process only requests to our host
    if request.uri().host() != Some(host) {
        return;
    }
    // pass the request to the ui service
    use tower::Service;
    let Ok(response) = ui::service().call(request).await else { return };
    // proceed only if OK
    if response.status().as_u16() != 200 {
        return;
    }
    let promise = pwrs_sw::axum_response_into_promise(response);
    event.respond_with(&promise).unwrap();
}
