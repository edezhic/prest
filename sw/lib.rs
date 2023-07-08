extern crate console_error_panic_hook;
mod utils;

// wasm_bindgen macro generates wasm-js bindings for the fetch event listener
#[wasm_bindgen::prelude::wasm_bindgen]
pub async fn service(host: &str, fetch_event: web_sys::FetchEvent) {
    use tower_service::Service;
    console_error_panic_hook::set_once();
    // convert web_sys::Request into http::Request with axum::Body
    let request = utils::axum_request_from_websys(fetch_event.request()).await;
    // process only requests to our host without query parameters
    if request.uri().host() != Some(host) || request.uri().query().is_some() {
        return;
    }
    // pass the request to the ui service
    let Ok(response) = ui::service().call(request).await else { return };
    // proceed only if OK
    if response.status().as_u16() != 200 {
        return;
    }
    // convert http::Request with axum::Body into Future<web_sys::Request>
    let response = utils::axum_response_to_websys(response);
    // convert future into promise
    let response = wasm_bindgen_futures::future_to_promise(response);
    fetch_event.respond_with(&response).unwrap();
}
