#![feature(let_chains)]

mod templates;

use pwrs::*;

pub fn ui_service() -> Router {
    use templates::*;
    Router::new()
        .route("/", render!(home))
        .layer(Htmxify::wrap(full_html))
}

#[cfg(host)]
#[derive(rust_embed::RustEmbed, Clone, Copy)]
#[folder = "./pub"]
struct Assets;

#[cfg(host)]
pub fn service() -> Router {
    Router::new()
        .merge(ui_service())
        .layer(pwrs_host::embed(Assets))
        .layer(pwrs_host::http_tracing())
}

#[cfg(sw)]
// wasm_bindgen macro generates wasm-js bindings for the fetch event listener
#[wasm_bindgen::prelude::wasm_bindgen]
pub async fn serve(host: &str, event: web_sys::FetchEvent) {
    pwrs_sw::set_panic_hook();
    let request = pwrs_sw::fetch_into_axum_request(&event).await;
    // process only requests to our host
    if request.uri().host() != Some(host) {
        return;
    }
    // pass the request through the ui service
    let Ok(response) = ui_service().call(request).await else { return };
    // proceed only if OK
    if response.status().as_u16() != 200 {
        return;
    }
    let promise = pwrs_sw::axum_response_into_promise(response);
    event.respond_with(&promise).unwrap();
}
