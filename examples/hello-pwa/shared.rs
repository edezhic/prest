use prest::*;

pub fn routes() -> Router {
    Router::new().route(
        "/",
        get(html!(
                (Head::example().pwa())
                body { 
                    h1{"Hello from PWA!"} 
                    (Scripts::default().with_sw())
                }
        )),
    )
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn handle_fetch(sw: ServiceWorkerGlobalScope, fe: FetchEvent) {
    serve(routes(), sw, fe).await
}
