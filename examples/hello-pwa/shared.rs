use prest::*;

pub fn routes() -> Router {
    Router::new().route(
        "/",
        get(html!(
                (Head::example("Hello PWA"))
                body { 
                    h1{"Hello from PWA!"} 
                    (Scripts::default())
                }
        )),
    )
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn handle_fetch(sw: ServiceWorkerGlobalScope, fe: FetchEvent) {
    serve(routes(), sw, fe).await
}
