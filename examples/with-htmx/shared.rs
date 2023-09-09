pub mod ui;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub async fn serve(host: &str, event: prest::sw::FetchEvent) {
    prest::sw::process_fetch_event(ui::service, host, event).await
}