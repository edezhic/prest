#![allow(dead_code)]

fn shared_routes() -> prest::Router {
    prest::Router::new().route(
        "/",
        prest::get(|| async {
            prest::maud_to_response(
                maud::html!((prest::head("Hello world PWA", None)) body {"Hello world!"}),
            )
        }),
    )
}

#[cfg(feature = "host")]
#[derive(rust_embed::RustEmbed, Clone)]
#[folder = "$OUT_DIR/assets"]
struct Assets;

#[cfg(feature = "host")]
#[tokio::main]
async fn main() {
    let service = shared_routes().layer(prest::host::embed(Assets));
    prest::host::serve(service, Default::default()).await.unwrap();
}

#[cfg(feature = "sw")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub async fn serve(sw: prest::sw::ServiceWorkerGlobalScope, event: prest::sw::FetchEvent) {
    prest::sw::process_fetch_event(shared_routes, sw, event).await
}
