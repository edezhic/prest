#![allow(dead_code)]

mod ui;

#[cfg(feature = "host")]
#[derive(rust_embed::RustEmbed, Clone)]
#[folder = "$OUT_DIR/assets"]
struct Assets;

#[cfg(feature = "host")]
#[derive(rust_embed::RustEmbed, Clone)]
#[folder = "./icons"]
struct Icons;

#[cfg(feature = "host")]
#[tokio::main]
async fn main() {
    let service = ui::service().layer(prest::host::embed(Assets)).layer(prest::host::embed(Icons));
    prest::host::serve(service, 80).await.unwrap();
}

#[cfg(feature = "sw")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub async fn serve(sw: prest::sw::ServiceWorkerGlobalScope, event: prest::sw::FetchEvent) {
    prest::sw::process_fetch_event(ui::service, sw, event).await
}
