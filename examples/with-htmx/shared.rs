pub mod ui;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub async fn serve(host: &str, event: pwrs_sw::FetchEvent) {
    pwrs_sw::process_fetch_event(ui::service, host, event).await
}