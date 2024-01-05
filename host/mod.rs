use crate::*;

mod auth;
pub use auth::*;
mod state;
mod traces;
use traces::*;

pub use tokio::{
    net::TcpListener,
    runtime::{Handle as RuntimeHandle, Runtime, RuntimeFlavor, Builder as RuntimeBuilder},
    sync::{Mutex, OnceCell, RwLock},
    task::block_in_place,
};

pub(crate) use gluesql::sled_storage::SledStorage as PersistentStorage;

/// Utility trait to use Router as the host
pub trait HostUtils {
    /// Init env vars, DB, auth, tracing, other utils and start the server
    fn run(self);
    fn serve(self);
    fn read_env(self) -> Self;
    fn init_tracing(self) -> Self;
    fn init_auth(self) -> Self;
    fn add_utility_layers(self) -> Self;
}

impl HostUtils for Router {
    fn run(self) {
        self.read_env()
            .init_auth()
            .init_tracing()
            .embed(DefaultAssets) // TODO: what to do about these?
            .add_utility_layers()
            .serve()
    }
    fn serve(self) {
        Runtime::new()
            .unwrap()
            .block_on(async move {
                let addr = check_port();
                let listener = TcpListener::bind(addr).await.unwrap();
                info!("Starting serving at {}", addr);
                axum::serve(listener, self).await
            })
            .unwrap();
    }
    fn read_env(self) -> Self {
        if let Err(e) = dotenvy::dotenv() {
            warn!(".env not used: {e}")
        }
        self
    }
    fn init_auth(self) -> Self {
        let (auth_layer, auth_routes) = init_auth();
        self.merge(auth_routes).layer(auth_layer)
    }
    fn init_tracing(self) -> Self {
        init_tracing_subscriber();
        self.layer(trace_layer())
    }
    fn add_utility_layers(self) -> Self {
        use tower_http::catch_panic::CatchPanicLayer;
        use tower_livereload::LiveReloadLayer;
        let host_services = ServiceBuilder::new().layer(CatchPanicLayer::custom(handle_panic));
        #[cfg(debug_assertions)]
        let host_services =
            host_services.layer(LiveReloadLayer::new().request_predicate(not_htmx_predicate));
        #[cfg(not(debug_assertions))]
        let host_services = host_services
            .layer(tower_http::compression::CompressionLayer::new())
            .layer(tower_http::limit::RequestBodyLimitLayer::new(
                request_body_limit(),
            ));
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

use std::net::SocketAddr;
fn check_port() -> SocketAddr {
    let port = if let Ok(v) = env::var("PORT") {
        v.parse::<u16>().unwrap_or(80)
    } else {
        80
    };
    SocketAddr::from(([0, 0, 0, 0], port))
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
