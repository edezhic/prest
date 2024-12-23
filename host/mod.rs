use crate::*;

mod admin;
mod remote;
mod server;
mod state;

mod docker;
pub(crate) use docker::*;

mod runtime;
pub use runtime::*;

mod system_info;
pub use system_info::SYSTEM_INFO;

mod sse;
pub use sse::*;

#[cfg(feature = "auth")]
mod auth;
#[cfg(feature = "auth")]
pub use auth::*;

#[cfg(feature = "traces")]
mod logs;
#[cfg(feature = "traces")]
use logs::*;
pub use logs::{init_tracing_subscriber, DEBUG, ERROR, INFO, TRACE, WARN};

#[cfg(all(feature = "traces", feature = "db"))]
pub(crate) mod analytics;

#[cfg(feature = "webview")]
mod webview;

pub use async_io::block_on as await_blocking;
pub use directories::*;
pub use dotenvy::dotenv;
pub use tokio::{
    io,
    net::ToSocketAddrs,
    runtime::{Builder as RuntimeBuilder, Handle as RuntimeHandle, Runtime, RuntimeFlavor},
    signal as tokio_signal,
    sync::{Mutex, OnceCell, RwLock},
    task::{block_in_place, JoinSet},
    time::{sleep, timeout},
};

#[cfg(feature = "db")]
mod sled;
#[cfg(feature = "db")]
pub(crate) use sled::SharedSledStorage as PersistentStorage;

state!(RT: PrestRuntime = { PrestRuntime::init() });
state!((crate) IS_REMOTE: bool = { env::var("DEPLOYED_TO_REMOTE").is_ok() });

/// Utility trait to use Router as the host
pub trait HostUtils {
    /// Init env vars, DB, auth, tracing, other utils and start the server
    fn run(self);
    fn serve(self);
    fn add_utility_layers(self) -> Self;
    fn add_default_assets(self) -> Self;
    fn add_analytics(self) -> Self;
    fn add_auth(self) -> Self;
}

impl HostUtils for Router {
    #[cfg(not(feature = "webview"))]
    fn run(self) {
        #[cfg(feature = "auth")]
        let admin = admin::routes().layer(axum::middleware::from_fn(check_admin));
        #[cfg(not(feature = "auth"))]
        let admin = admin::routes();
        self.route("/health", get(StatusCode::OK))
            .add_auth()
            .add_default_assets()
            .add_analytics()
            .nest("/admin", admin)
            .add_utility_layers()
            .serve()
    }
    #[cfg(feature = "webview")]
    fn run(self) {
        std::thread::spawn(|| self.read_env().add_default_favicon().serve());
        webview::init_webview(&localhost(&check_port())).expect("Webview must initialize");
    }
    fn serve(self) {
        RT.block_on(async move { server::start(self).await })
            .expect("Server should stop gracefully");
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
    fn add_analytics(self) -> Self {
        #[cfg(feature = "traces")]
        return self.layer(analytics::AnalyticsLayer::init());
        #[cfg(not(feature = "traces"))]
        self
    }
    fn add_default_assets(mut self) -> Self {
        embed_build_output_as!(BuiltinAssets);
        self = self.embed(BuiltinAssets);

        // add default favicon
        let current_resp = RT
            .block_on(async {
                self.call(
                    Request::get("/favicon.ico")
                        .body(Body::empty())
                        .expect("Proper request"),
                )
                .await
            })
            .expect("Should be infallible");
        if current_resp.status() == 404 {
            self = self.route("/favicon.ico", get(|| async {
                ([(header::CACHE_CONTROL, "max-age=360000, stale-while-revalidate=8640000, stale-if-error=60480000")], Body::from(FAVICON))
            }));
        }
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

#[cfg(feature = "auth")]
async fn check_admin(user: User, request: Request, next: Next) -> Result<Response> {
    if user.is_admin() {
        Ok(next.run(request).await)
    } else {
        Err(Error::Unauthorized)
    }
}

fn handle_panic(err: Box<dyn std::any::Any + Send + 'static>) -> Response {
    let details = get_panic_message(err);

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

fn get_panic_message(err: Box<dyn std::any::Any + Send + 'static>) -> String {
    if let Some(s) = err.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = err.downcast_ref::<&str>() {
        s.to_string()
    } else {
        "Unknown panic".to_string()
    }
}

#[allow(dead_code)]
const DEFAULT_REQUEST_BODY_LIMIT: usize = 10_000_000;
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

const INTERNAL_PATHS: [&str; 5] = [
    "/tower-livereload",
    "/default-view-transition",
    "/admin",
    "/sw/health",
    "/prest.js",
];
fn internal_request(request: &Request) -> bool {
    let path = request.uri().path();
    for internal in INTERNAL_PATHS {
        if path.starts_with(internal) {
            return true;
        }
    }
    false
}
