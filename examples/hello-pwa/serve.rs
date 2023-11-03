use prest::*;

pub fn shared() -> Router {
    Router::new().route("/", get(html!(
            (Head::default().webmanifest("/.webmanifest")) 
            body { h1{"Hello from PWA!"} (Scripts::default().with_sw())}
    )))
}

#[cfg(feature = "sw")]
#[wasm_bindgen]
pub async fn handle_fetch(sw: ServiceWorkerGlobalScope, fe: FetchEvent) {
    serve(shared(), sw, fe).await
}

#[cfg(feature = "host")]
#[tokio::main]
pub async fn main() {
    include_build_output_as!(Dist);
    serve(shared().embed(Dist), Default::default()).await
}
