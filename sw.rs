use crate::*;

pub use console_error_panic_hook::set_once as set_panic_hook;
use js_sys::{Array, Reflect, Set, Uint8Array, Promise};
pub use web_sys::{FetchEvent, ServiceWorkerGlobalScope, console};
use wasm_bindgen::{JsCast, JsValue};

pub use wasm_bindgen::prelude::wasm_bindgen;

/// Process request to the same host using the router and respond if status < 400
pub async fn serve(mut router: Router, sw: ServiceWorkerGlobalScope, event: FetchEvent) {
    set_panic_hook();
    
    let host = &sw.location().host();
    
    let request = fetch_into_axum_request(&event).await;
    // process only requests to our host
    if request.uri().host() != Some(host) {
        return;
    }

    // pass the request to the router
    let response = match router.call(request).await {
        Ok(res) => res,
        Err(e) => {
            #[cfg(debug_assertions)]
            console::log_3(&"SW error while processing ".into(), &event.request(), &format!(" : {:?}", e).into());
            return 
        }
    };
    
    if response.status().as_u16() >= 400 {    
        return
    }
    
    let response = axum_response_to_websys(response);
    let promise = wasm_bindgen_futures::future_to_promise(response);
    event.respond_with(&promise).unwrap();
}

pub async fn fetch_into_axum_request(fetch_event: &web_sys::FetchEvent) -> http::Request<Body> {
    let fetch_request = fetch_event.request();

    // initialize a new request builder with method and url
    let mut request = http::request::Builder::new()
        .method(fetch_request.method().as_str())
        .uri(fetch_request.url());

    // collect web_sys::Headers items into the request
    let headers = Set::new(&fetch_request.headers()).entries();
    while let Ok(item) = headers.next() {
        if Reflect::get(&item, &JsValue::from("done"))
            .unwrap()
            .is_truthy()
        {
            break;
        }
        let pair = Reflect::get(&item, &JsValue::from("value"))
            .unwrap()
            .unchecked_into::<Array>()
            .at(0)
            .unchecked_into::<Array>();
        let (key, value) = (
            pair.at(0).as_string().unwrap(),
            pair.at(1).as_string().unwrap(),
        );
        request = request.header(key, value);
    }

    let Some(stream) = fetch_request.body() else {
        return request.body(Body::empty()).unwrap()
    };
    let stream = stream
        .get_reader()
        .unchecked_into::<web_sys::ReadableStreamDefaultReader>();
    let mut buf = vec![];
    // collect js body stream into a buffer
    while let Ok(item) = wasm_bindgen_futures::JsFuture::from(stream.read()).await {
        let done = Reflect::get(&item, &JsValue::from("done"))
            .unwrap()
            .is_truthy();
        let mut data = Reflect::get(&item, &JsValue::from("value"))
            .unwrap()
            .unchecked_into::<Uint8Array>()
            .to_vec();
        buf.append(&mut data);
        if done {
            break;
        }
    }
    request.body(Body::from(buf)).unwrap()
}

pub async fn axum_response_to_websys(response: http::Response<Body>) -> Result<JsValue, JsValue> {
    // init web_sys::Headers
    let websys_response_headers = web_sys::Headers::new()?;
    // collect http::HeaderMap into web_sys::Headers
    for (name, value) in response.headers() {
        let Ok(value) = std::str::from_utf8(value.as_bytes()) else { continue };
        websys_response_headers
            .append(name.as_str(), value)
            .unwrap();
    }
    // init web_sys::ResponseInit (~= http::response::Parts)
    let mut parts = web_sys::ResponseInit::new();
    parts
        .headers(&websys_response_headers)
        .status(response.status().as_u16());

    // collect axum::Body into a buffer
    let body = response.into_body();
    let mut buf = Vec::with_capacity(body.size_hint().lower() as usize);
    let mut body_stream = body.into_data_stream();
    while let Some(chunk) = body_stream.next().await {
        bytes::BufMut::put(&mut buf, chunk.unwrap());
    }
    let body = Some(buf.as_mut_slice());

    // initialize web_sys::Response with body and parts
    let websys_response = web_sys::Response::new_with_opt_u8_array_and_init(body, &parts)?;

    // convert into JsValue
    Ok(websys_response.into())
}