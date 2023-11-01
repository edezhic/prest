use prest::*;

pub fn shared() -> Router {
    Router::new().route("/", get(html!((Head::pwa()) h1{"Hello from PWA!"})))
}

#[cfg(feature = "sw")]
#[wasm_bindgen]
pub async fn handle_fetch(sw: ServiceWorkerGlobalScope, fe: FetchEvent) {
    serve(shared(), sw, fe).await
}

#[cfg(feature = "host")]
#[tokio::main]
pub async fn main() {
    #[derive(Embed)] struct Dist;
    serve(shared().embed::<Dist>(), Default::default()).await
}
