#![allow(dead_code)]

mod pages;

#[cfg(feature = "host")]
#[derive(rust_embed::RustEmbed, Clone)]
#[folder = "$OUT_DIR/assets"]
struct Assets;

#[cfg(feature = "host")]
#[tokio::main]
async fn main() {
    let service = pages::service().layer(prest::host::embed(Assets));
    prest::host::serve(service, Default::default()).await.unwrap();
}

#[cfg(feature = "sw")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub async fn serve(sw: prest::sw::ServiceWorkerGlobalScope, event: prest::sw::FetchEvent) {
    prest::sw::process_fetch_event(pages::service, sw, event).await
}
