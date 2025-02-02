use std::future::Future;

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
pub(crate) use system_info::SystemStat;
pub use system_info::SYSTEM_INFO;

mod sse;
pub use sse::*;

#[cfg(feature = "auth")]
pub(crate) mod auth;
#[cfg(feature = "auth")]
pub use auth::*;

#[cfg(feature = "traces")]
#[doc(hidden)]
pub mod logs;
#[cfg(feature = "traces")]
use logs::*;

#[cfg(feature = "db")]
pub(crate) mod db;
// #[cfg(feature = "db")]
// pub use db;

#[cfg(all(feature = "traces", feature = "db"))]
pub(crate) mod analytics;

#[cfg(feature = "webview")]
mod webview;

pub(crate) use directories::*;
#[doc(hidden)]
pub use dotenvy::dotenv;
pub use tokio::{
    spawn,
    sync::{Mutex, RwLock},
    task::{block_in_place, JoinSet},
    time::{sleep, timeout},
};

#[doc(hidden)]
pub mod _host {
    pub use tokio::runtime::{
        Builder as RuntimeBuilder, Handle as RuntimeHandle, Runtime, RuntimeFlavor,
    };
}

state!(RT: PrestRuntime = { PrestRuntime::init() });
state!((crate) IS_REMOTE: bool = { env_var("DEPLOYED_TO_REMOTE").is_ok() });

/// Utility trait to use Router as the host
#[async_trait]
pub trait HostUtils: Sized {
    /// Init env vars, DB, auth, tracing, other utils and start the server
    async fn run(self) -> Result;
    async fn serve(self) -> Result;
    fn add_utility_layers(self) -> Self;
    async fn add_default_assets(self) -> Self;
    fn add_analytics(self) -> Self;
    fn add_auth(self) -> Result<Self>;
}

#[async_trait]
impl HostUtils for Router {
    #[cfg(not(feature = "webview"))]
    async fn run(self) -> Result {
        #[cfg(feature = "auth")]
        let admin = admin::routes()
            .await
            .layer(axum::middleware::from_fn(check_admin));
        #[cfg(not(feature = "auth"))]
        let admin = admin::routes().await;
        self.route("/health", get(StatusCode::OK))
            .add_auth()?
            .add_default_assets()
            .await
            .add_analytics()
            .nest("/admin", admin)
            .add_utility_layers()
            .serve()
            .await?;
        OK
    }
    #[cfg(feature = "webview")]
    async fn run(self) {
        std::thread::spawn(|| {
            RT.block_on(async {
                self.add_default_assets()
                    .await
                    .add_utility_layers()
                    .serve()
                    .await
                    .expect("Server should shutdown gracefully")
            })
        });
        webview::init_webview(&localhost(&check_port())).expect("Webview must initialize");
        OK
    }
    async fn serve(self) -> Result {
        server::start(self).await
    }

    fn add_auth(self) -> Result<Self> {
        #[cfg(feature = "auth")]
        {
            let (auth_layer, auth_routes) = auth::init_auth_module()?;
            Ok(self.merge(auth_routes).layer(auth_layer))
        }
        #[cfg(not(feature = "auth"))]
        Ok(self)
    }
    fn add_analytics(self) -> Self {
        #[cfg(feature = "traces")]
        return self.layer(analytics::AnalyticsLayer::init());
        #[cfg(not(feature = "traces"))]
        self
    }
    async fn add_default_assets(mut self) -> Self {
        #[derive(Embed)]
        #[folder = "$OUT_DIR"]
        pub struct BuiltinAssets;
        self = self.embed(BuiltinAssets);

        use tower::Service;
        // add default favicon
        let current_resp = self
            .call(
                Request::get("/favicon.ico")
                    .body(Body::empty())
                    .expect("Proper request"),
            )
            .await
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
        let host_services =
            tower::ServiceBuilder::new().layer(CatchPanicLayer::custom(handle_panic));
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
    if let Ok(v) = env_var("REQUEST_BODY_LIMIT") {
        v.parse::<usize>().unwrap_or(DEFAULT_REQUEST_BODY_LIMIT)
    } else {
        DEFAULT_REQUEST_BODY_LIMIT
    }
}

#[allow(dead_code)]
fn not_htmx_predicate<Body>(req: &Request<Body>) -> bool {
    !req.headers().contains_key("hx-request")
}

const INTERNAL_PATHS: [&str; 2] = ["/tower-livereload", "/sw/health"];
fn internal_request(request: &Request) -> bool {
    let path = request.uri().path();
    for internal in INTERNAL_PATHS {
        if path.starts_with(internal) {
            return true;
        }
    }
    false
}

impl<T> NonSendFuturesAdapter for T where T: Future {}
pub trait NonSendFuturesAdapter: Future {
    fn await_blocking<R>(self) -> R
    where
        Self: Future<Output = R> + Sized,
    {
        tokio::task::block_in_place(move || tokio::runtime::Handle::current().block_on(self))
    }
}
