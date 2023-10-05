#![allow(dead_code)]
use prest::*;

fn shared_routes() -> Router {
    Router::new().route("/", get(|| async { maud_to_response(
        maud::html!(
            (maud_pwa_head("Prest app with tracing", Some(maud::html!(link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/water.css@2/out/dark.css"{}))))
            body {
                h1{"Progressive RESTful application with tracing (check out the terminal!)"}
            }
        )
    )}))
}

#[cfg(feature = "host")]
#[derive(rust_embed::RustEmbed, Clone)]
#[folder = "$OUT_DIR/assets"]
struct Assets;

#[cfg(feature = "host")]
#[tokio::main]
async fn main() {
    start_printing_traces();
    let service = shared_routes()
        .layer(embed(Assets))
        .layer(http_tracing());
    serve(service, Default::default()).await.unwrap();
}

#[cfg(feature = "sw")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub async fn serve(sw: sw::ServiceWorkerGlobalScope, event: sw::FetchEvent) {
    sw::process_fetch_event(shared_routes, sw, event).await
}
