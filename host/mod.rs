use crate::*;

mod admin;
use self::admin::db_routes;

mod state;

mod ssh;
pub use ssh::*;

mod docker;
pub use docker::*;

mod analytics;
pub use analytics::*;

mod shutdown;
pub use shutdown::*;

mod schedule;
pub use schedule::*;

#[cfg(feature = "auth")]
mod auth;
#[cfg(feature = "auth")]
pub use auth::*;

#[cfg(feature = "https")]
mod https;

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
use std::net::SocketAddr;
pub use tokio::{
    io,
    net::ToSocketAddrs,
    runtime::{Builder as RuntimeBuilder, Handle as RuntimeHandle, Runtime, RuntimeFlavor},
    sync::{Mutex, OnceCell, RwLock},
    task::block_in_place,
};
pub type SseItem = Result<SseEvent, Infallible>;

#[cfg(feature = "db")]
pub(crate) use gluesql::sled_storage::SledStorage as PersistentStorage;

/// Utility trait to use Router as the host
pub trait HostUtils {
    /// Init env vars, DB, auth, tracing, other utils and start the server
    fn run(self);
    fn serve(self);
    fn add_utility_layers(self) -> Self;
    fn add_default_embeddings(self) -> Self;
    fn add_analytics(self) -> Self;
    fn add_tracing(self) -> Self;
    fn add_auth(self) -> Self;
    fn add_admin(self) -> Self;
}

impl HostUtils for Router {
    #[cfg(not(feature = "webview"))]
    fn run(self) {
        self.route("/health", get(StatusCode::OK))
            .route("/shutdown", get(|| async { SHUTDOWN.initiate() }))
            .add_auth()
            .add_admin()
            .add_tracing()
            .add_analytics()
            .add_default_embeddings()
            .add_utility_layers()
            .serve()
    }
    #[cfg(feature = "webview")]
    fn run(self) {
        std::thread::spawn(|| {
            self.read_env()
                .add_tracing()
                .add_default_embeddings()
                .serve()
        });
        webview::init_webview(&localhost(&check_port())).unwrap();
    }
    fn serve(self) {
        Runtime::new()
            .unwrap()
            .block_on(async move {
                #[cfg(any(not(feature = "https"), debug))]
                {
                    let port = check_port();
                    let addr = SocketAddr::from(([0, 0, 0, 0], port));
                    #[cfg(debug_assertions)]
                    info!("Starting serving at {}", localhost(port));
                    #[cfg(not(debug_assertions))]
                    info!("Starting serving at {addr}");
                    axum_server::bind(addr)
                        .handle(SHUTDOWN.new_server_handle())
                        .serve(self.into_make_service())
                        .await
                }
                #[cfg(all(feature = "https", release))]
                https::serve_https().await
            })
            .unwrap();
    }
    fn add_admin(mut self) -> Self {
        self = self.merge(db_routes());
        self.route(
            "/deploy",
            get(|| async {
                tokio::spawn(async {
                    let project_path = "/Users/egordezic/Desktop/prest";
                    let target_path = "/Users/egordezic/Desktop/prest/target";
                    let binary_path = build_linux_binary(project_path, target_path).unwrap();
                    remote_update(&binary_path).await.unwrap();
                });
            }),
        )
        .route("/admin", get(admin::page))
    }
    fn add_analytics(self) -> Self {
        self.layer(AnalyticsLayer::init())
    }
    fn add_auth(self) -> Self {
        #[cfg(feature = "auth")]
        {
            let (auth_layer, auth_routes) = auth::init_auth_module();
            self.merge(auth_routes).layer(auth_layer)
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
    fn add_default_embeddings(self) -> Self {
        #[cfg(feature = "embed")]
        return self.embed(DefaultAssets); // TODO: what to do about these?
        #[cfg(not(feature = "embed"))]
        self
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

fn check_port() -> u16 {
    if let Ok(v) = env::var("PORT") {
        v.parse::<u16>().unwrap_or(80)
    } else {
        80
    }
}

#[allow(dead_code)]
fn localhost(port: u16) -> String {
    format!(
        "http://localhost{}",
        if port == 80 {
            "".to_owned()
        } else {
            format!(":{port}")
        }
    )
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
fn internal_req(request: &Request) -> bool {
    let path = request.uri().path();
    for internal in INTERNAL_PATHS {
        if path.starts_with(internal) {
            return true;
        }
    }
    false
}
