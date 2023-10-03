#![allow(dead_code)]

mod pages;
use prest::*;

#[cfg(feature = "host")]
#[derive(rust_embed::RustEmbed, Clone)]
#[folder = "$OUT_DIR/assets"]
struct Assets;

#[cfg(feature = "host")]
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let service = pages::service().layer(embed(Assets));
    serve(service, Default::default()).await.unwrap();
}

#[cfg(feature = "sw")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub async fn serve(sw: sw::ServiceWorkerGlobalScope, event: sw::FetchEvent) {
    sw::process_fetch_event(pages::service, sw, event).await
}
