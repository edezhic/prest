#![allow(dead_code)]
use prest::*;

fn shared_routes() -> Router {
    Router::new().route(
        "/",
        get(|| async {
            maud_to_response(
                maud::html!((maud_head("Hello world PWA", None)) body {"Hello world!"}),
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
    let service = shared_routes().layer(middleware::embed(Assets));
    serve(service, Default::default()).await.unwrap();
}

#[cfg(feature = "sw")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub async fn serve(sw: sw::ServiceWorkerGlobalScope, event: sw::FetchEvent) {
    sw::process_fetch_event(shared_routes, sw, event).await
}
