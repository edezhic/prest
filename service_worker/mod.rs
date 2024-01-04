use crate::*;
pub use console_error_panic_hook::set_once as set_panic_hook;
use js_sys::{Array, Promise, Reflect, Set, Uint8Array};
use std::sync::Mutex;
use tracing_subscriber::fmt::{format::{Pretty, FmtSpan}, time::UtcTime};
use tracing_subscriber::prelude::*;
use tracing_web::{performance_layer, MakeWebConsoleWriter};
use wasm_bindgen::{JsCast, JsValue};
pub use web_sys::{console, FetchEvent, ServiceWorkerGlobalScope};

pub use wasm_bindgen::prelude::wasm_bindgen;

// TODO: figure out how to use gluesql::idb_storage::IdbStorage as PersistentStorage for SW
pub(crate) type PersistentStorage = gluesql::shared_memory_storage::SharedMemoryStorage;

#[macro_export]
macro_rules! state {
    ($struct_name:ident: $type:ty = $init:block) => {
        pub static $struct_name: prest::Lazy<$type> = prest::Lazy::new(|| {
            fn init() -> Result<$type, Box<dyn std::error::Error>> {
                let v = { $init };
                Ok(v)
            }
            init().unwrap()
        });
    };
    ($struct_name:ident: $type:ty = async $init:block) => {
        pub static $struct_name: prest::Lazy<$type> = prest::Lazy::new(|| {
            async fn init() -> Result<$type, Box<dyn std::error::Error>> {
                let v = { $init };
                Ok(v)
            }
            prest::block_on(init()).unwrap()
        });
    };
}

pub struct ServeOptions {
    //pub log_filter: LevelFilter,
    pub embed_default_assets: bool,
}
impl Default for ServeOptions {
    fn default() -> Self {
        Self {
            //log_filter: LevelFilter::DEBUG,
            embed_default_assets: true,
        }
    }
}

/// Util trait to add serve function to the Router
pub trait Serving {
    fn serve(self);
    fn serve_with_opts(self, opts: ServeOptions);
}

static mut ROUTER: Option<Router> = None;

impl Serving for Router {
    fn serve(self) {
        self.serve_with_opts(Default::default())
    }
    fn serve_with_opts(mut self, opts: ServeOptions) {
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_ansi(false) // Only partially supported across browsers
            .with_timer(UtcTime::rfc_3339())
            .with_writer(tracing_web::MakeWebConsoleWriter::new().with_pretty_level()) 
            .with_level(false)
            .with_span_events(FmtSpan::ACTIVE); 
        let perf_layer = performance_layer()
            .with_details_from_fields(Pretty::default());

        tracing_subscriber::registry()
            .with(fmt_layer)
            .with(perf_layer)
            .init(); // Install these as subscribers to tracing events

        self = self.route("/sw/health", get(StatusCode::OK));

        // self = self.layer(tower_http::trace::TraceLayer::new_for_http());
        // panicked at library/std/src/sys/wasm/../unsupported/time.rs:13:9:
        // "time not implemented on this platform"
        unsafe { ROUTER = Some(self) }
    }
}

/// Process request to the same host using the router and respond if status < 400
#[wasm_bindgen]
pub async unsafe fn handle_fetch(sw: ServiceWorkerGlobalScope, event: FetchEvent) {
    let Some(ref mut router) = ROUTER else {
        return;
    };

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
            console::log_3(
                &"SW error while processing ".into(),
                &event.request(),
                &format!(" : {:?}", e).into(),
            );
            return;
        }
    };

    if response.status().as_u16() >= 400 {
        return;
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
        return request.body(Body::empty()).unwrap();
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
        let Ok(value) = std::str::from_utf8(value.as_bytes()) else {
            continue;
        };
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
    let mut body = axum::body::to_bytes(body, 1000000).await.unwrap().to_vec();
    let body = Some(body.as_mut_slice());

    // initialize web_sys::Response with body and parts
    let websys_response = web_sys::Response::new_with_opt_u8_array_and_init(body, &parts)?;

    // convert into JsValue
    Ok(websys_response.into())
}
