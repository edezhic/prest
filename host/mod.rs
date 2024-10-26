use crate::*;

mod admin;
pub use admin::*;

mod state;

mod server;

mod ssh;
pub(crate) use ssh::*;

mod docker;
pub(crate) use docker::*;

mod shutdown;
pub use shutdown::*;

mod schedule;
pub use schedule::*;

#[cfg(feature = "auth")]
mod auth;
#[cfg(feature = "auth")]
pub use auth::*;

#[cfg(feature = "traces")]
mod traces;
pub use traces::init_tracing_subscriber;
#[cfg(feature = "traces")]
use traces::*;

#[cfg(feature = "webview")]
mod webview;

pub use axum::response::sse::{Event as SseEvent, KeepAlive as SseKeepAlive, Sse};
pub use directories::*;
pub use dotenvy::dotenv;
pub use tokio::{
    io,
    net::ToSocketAddrs,
    runtime::{Builder as RuntimeBuilder, Handle as RuntimeHandle, Runtime, RuntimeFlavor},
    sync::{Mutex, OnceCell, RwLock},
    task::block_in_place,
};

/// Alias for Server Sent Events event
pub type SseItem = Result<SseEvent, Infallible>;

#[cfg(feature = "db")]
pub(crate) use gluesql::gluesql_sled_storage::SledStorage as PersistentStorage;

use std::sync::atomic::AtomicUsize;
pub struct PrestRuntime {
    pub inner: Runtime,
    pub running_scheduled_tasks: AtomicUsize,
}

state!(RT: PrestRuntime = { PrestRuntime { inner: Runtime::new().unwrap(), running_scheduled_tasks: 0.into() } });

/// Utility trait to use Router as the host
pub trait HostUtils {
    /// Init env vars, DB, auth, tracing, other utils and start the server
    fn run(self);
    fn serve(self);
    fn add_utility_layers(self) -> Self;
    fn add_default_favicon(self) -> Self;
    fn add_tracing(self) -> Self;
    fn add_auth(self) -> Self;
    fn add_admin(self) -> Self;
}

impl HostUtils for Router {
    #[cfg(not(feature = "webview"))]
    fn run(self) {
        self.route("/health", get(StatusCode::OK))
            .add_auth()
            .add_admin()
            .add_tracing()
            .add_default_favicon()
            .add_utility_layers()
            .serve()
    }
    #[cfg(feature = "webview")]
    fn run(self) {
        std::thread::spawn(|| self.read_env().add_tracing().add_default_favicon().serve());
        webview::init_webview(&localhost(&check_port())).unwrap();
    }
    fn serve(self) {
        RT.inner
            .block_on(async move { server::start(self).await })
            .unwrap();
    }

    fn add_admin(mut self) -> Self {
        self = self.merge(db_routes());
        self.route("/admin", get(admin::page))
            .route("/admin/deploy", get(admin::deploy))
            .route("/admin/logs", get(admin::logs))
            .route("/admin/debug_logs", get(admin::debug_logs))
            .route("/admin/analytics", get(admin::analytics))
            .layer(admin::AnalyticsLayer::init())
    }

    fn add_auth(self) -> Self {
        #[cfg(feature = "auth")]
        {
            let (auth_layer, auth_routes) = auth::init_auth_module();
            self.merge(auth_routes)
                .layer(auth_layer)
                .route("/admin/shutdown", get(admin::shutdown))
        }
        #[cfg(not(feature = "auth"))]
        self
    }
    fn add_tracing(self) -> Self {
        #[cfg(feature = "traces")]
        return self.layer(trace_layer());
        #[cfg(not(feature = "traces"))]
        self
    }
    fn add_default_favicon(mut self) -> Self {
        let current_resp = RT
            .inner
            .block_on(async {
                self.call(Request::get("/favicon.ico").body(Body::empty()).unwrap())
                    .await
            })
            .unwrap();
        if current_resp.status() == 404 {
            self.route("/favicon.ico", get(|| async {
                ([(header::CACHE_CONTROL, "max-age=360000, stale-while-revalidate=8640000, stale-if-error=60480000")], Body::from(FAVICON))
            }))
        } else {
            self
        }
    }
    fn add_utility_layers(self) -> Self {
        use tower_http::catch_panic::CatchPanicLayer;
        let host_services = ServiceBuilder::new().layer(CatchPanicLayer::custom(handle_panic));
        #[cfg(debug_assertions)]
        let host_services = host_services
            .layer(tower_livereload::LiveReloadLayer::new().request_predicate(not_htmx_predicate));
        #[cfg(not(debug_assertions))]
        let host_services = host_services
            .layer(tower_http::compression::CompressionLayer::new())
            .layer(tower_http::limit::RequestBodyLimitLayer::new(
                request_body_limit(),
            ));

        let host_services = host_services
            .layer(tower_http::normalize_path::NormalizePathLayer::trim_trailing_slash());

        self.layer(host_services)
    }
}

fn handle_panic(err: Box<dyn std::any::Any + Send + 'static>) -> Response {
    let details = if let Some(s) = err.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = err.downcast_ref::<&str>() {
        s.to_string()
    } else {
        "Unknown panic message".to_string()
    };

    #[cfg(debug_assertions)]
    let body = format!("Panic: {details}");
    #[cfg(not(debug_assertions))]
    let body = format!("Internal error");

    error!("Panic occured: {details}");

    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(body))
        .unwrap()
}

#[allow(dead_code)]
const DEFAULT_REQUEST_BODY_LIMIT: usize = 1_000_000;
#[allow(dead_code)]
fn request_body_limit() -> usize {
    if let Ok(v) = env::var("REQUEST_BODY_LIMIT") {
        v.parse::<usize>().unwrap_or(DEFAULT_REQUEST_BODY_LIMIT)
    } else {
        DEFAULT_REQUEST_BODY_LIMIT
    }
}

#[allow(dead_code)]
fn not_htmx_predicate<Body>(req: &Request<Body>) -> bool {
    !req.headers().contains_key("hx-request")
}

const INTERNAL_PATHS: [&str; 3] = ["/tower-livereload", "/default-view-transition", "/admin"];
fn filter_request(request: &Request) -> bool {
    let path = request.uri().path();
    for internal in INTERNAL_PATHS {
        if path.starts_with(internal) {
            return true;
        }
    }
    false
}

fn filter_response(response: &Response) -> bool {
    let status = response.status();
    if [304, 404, 405].contains(&status.as_u16()) {
        return true;
    }
    false
}

use async_broadcast::{broadcast, Receiver, Sender};
// use stream::{Map, TryStream};

#[derive(Clone)]
pub struct SseEventWrapper<T: Clone + Send> {
    pub event_name: String,
    pub data: T,
}

// unsafe impl<T: Clone + Send> Send for SseEventWrapper<T> {}
pub struct SseBroadcast<T: Clone + Send> {
    sender: Sender<SseEventWrapper<T>>,
    receiver: Receiver<SseEventWrapper<T>>,
}

impl<T: Clone + Send> Default for SseBroadcast<T> {
    fn default() -> Self {
        let (sender, receiver) = broadcast(1000);
        Self { sender, receiver }
    }
}

impl<T: Clone + Send> SseBroadcast<T> {
    pub fn stream(&self) -> Receiver<SseEventWrapper<T>> {
        self.receiver.new_receiver()
    }

    pub async fn send<E: Into<String>>(&self, event_name: E, data: T) -> Result {
        self.sender
            .broadcast_direct(SseEventWrapper {
                event_name: event_name.into(),
                data,
            })
            .await
            .map_err(|e| anyhow!("{e}"))?;
        Ok(())
    }
}

pub trait SseBroadcastExt<T: Clone + Send> {
    fn stream_and_render<F>(&self, f: F) -> Response
    where
        F: FnMut(&String, T) -> Markup + std::marker::Send + 'static;

    // fn subscribe<F, S>(&self, f: F) -> MethodRouter<S> where
    // // S:
    // F: FnMut(&String, T) -> Markup + std::marker::Send + 'static;
}

impl<T: Clone + Send + 'static + std::marker::Sync> SseBroadcastExt<T> for SseBroadcast<T> {
    fn stream_and_render<F>(&self, mut f: F) -> Response
    where
        F: (FnMut(&String, T) -> Markup) + std::marker::Send + 'static,
    {
        let stream = self.receiver.new_receiver().map(move |event| {
            let event_name = event.event_name;
            let data = event.data;
            let rendered = f(&event_name, data);
            SseEvent::default().event(event_name).data(rendered.0)
        });

        Sse::new(stream.map(Ok::<axum::response::sse::Event, Infallible>))
            .keep_alive(SseKeepAlive::default())
            .into_response()
    }

    // fn subscribe<F, S>(&self, f: F) -> MethodRouter<S> where
    // // S:
    // F: FnMut(&String, T) -> Markup + std::marker::Send + 'static {
    //     get(|| async {})
    // }
}

// type X = MethodRo
