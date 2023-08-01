mod homepage;
use pwrs::*;

fn ui_service() -> Router {
    Router::new().route("/", render!(homepage))
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(rust_embed::RustEmbed, Clone, Copy)]
#[folder = "./pub"]
struct Assets;

#[cfg(not(target_arch = "wasm32"))]
pub fn service() -> Router {
    Router::new()
        .merge(ui_service())
        .layer(pwrs_host::embed(Assets))
        .layer(pwrs_host::http_tracing())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub async fn serve(host: &str, event: pwrs_sw::FetchEvent) {
    pwrs_sw::process_fetch_event(ui_service, host, event).await
}
