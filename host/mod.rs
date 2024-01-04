mod auth;
pub use auth::*;
use tower_sessions::{Expiry, SessionManagerLayer};

use crate::*;
use axum::body::Bytes;
use std::net::SocketAddr;
pub use tokio::{
    net::TcpListener,
    runtime::{Handle as RuntimeHandle, Runtime, RuntimeFlavor},
    sync::{Mutex, OnceCell, RwLock},
    task::block_in_place,
};
pub use tower::ServiceBuilder;
use tower_http::{
    body::Full,
    catch_panic::CatchPanicLayer,
    classify::{ServerErrorsAsFailures, SharedClassifier},
    compression::CompressionLayer,
    limit::RequestBodyLimitLayer,
    trace::{DefaultOnResponse, TraceLayer},
};
use tower_livereload::LiveReloadLayer;
use tracing::{Level, Span};
use tracing_subscriber::{
    filter::{self, LevelFilter},
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

pub(crate) use gluesql::sled_storage::SledStorage as PersistentStorage;

#[macro_export]
macro_rules! state {
    ($struct_name:ident: $type:ty = $init:block) => {
        pub static $struct_name: prest::Lazy<$type> = prest::Lazy::new(|| {
            fn init() -> Result<$type> {
                let v = { $init };
                Ok(v)
            }
            init().unwrap()
        });
    };
    ($struct_name:ident: $type:ty = async $init:block) => {
        pub static $struct_name: prest::Lazy<$type> = prest::Lazy::new(|| {
            async fn init() -> Result<$type> {
                let v = { $init };
                Ok(v)
            }
            if let Ok(handle) = prest::RuntimeHandle::try_current() {
                if handle.runtime_flavor() != prest::RuntimeFlavor::CurrentThread {
                    prest::block_in_place(move || handle.block_on(init()).unwrap())
                } else {
                    panic!("Prest doesn't support async state inside of the tokio's current_thread runtime yet")
                }
            } else {
                prest::Runtime::new().unwrap().block_on(init()).unwrap()
            }
        });
    };
}

/// Configuration for the server
pub struct ServeOptions {
    pub addr: SocketAddr,
    pub request_body_limit: usize,
    pub log_filter: LevelFilter,
    pub embed_default_assets: bool,
}
impl Default for ServeOptions {
    fn default() -> Self {
        let port = if let Ok(v) = std::env::var("PORT") {
            v.parse::<u16>().unwrap_or(80)
        } else {
            80
        };
        Self {
            addr: SocketAddr::from(([0, 0, 0, 0], port)),
            request_body_limit: 1_000_000,
            log_filter: LevelFilter::DEBUG,
            embed_default_assets: true,
        }
    }
}

/// Util trait to add run function to the Router
pub trait Serving {
    fn run(self);
    fn serve_with_opts(self, opts: ServeOptions);
}

/// Start tokio+hyper based server
impl Serving for Router {
    fn run(self) {
        self.serve_with_opts(ServeOptions::default())
    }
    fn serve_with_opts(mut self, opts: ServeOptions) {
        dotenvy::dotenv().unwrap();
        init_tracing(opts.log_filter);

        let host_services = ServiceBuilder::new().layer(CatchPanicLayer::new()).layer(
            TraceLayer::new_for_http()
                .on_eos(())
                .on_body_chunk(())
                .on_request(|request: &Request, _span: &Span| {
                    if internal_req(request) {
                        tracing::trace!("started {} {}", request.method(), request.uri().path())
                    } else {
                        tracing::debug!("started {} {}", request.method(), request.uri().path())
                    }
                })
                .on_response(DefaultOnResponse::new().level(Level::TRACE))
                // BROKEN ATM: https://github.com/tower-rs/tower-http/issues/432
                //.on_response(|response: &Response, latency: std::time::Duration, span: &Span| {
                //    tracing::debug!("response generated in {:?}", latency)
                //})
                .make_span_with(|request: &Request| {
                    let method = request.method().as_str();
                    let uri = request.uri().to_string();
                    if internal_req(request) {
                        return tracing::trace_span!("->", method, uri);
                    }

                    match *request.method() {
                        Method::GET => tracing::debug_span!("-> GET", uri),
                        Method::POST => tracing::debug_span!("-> POST", uri),
                        Method::PUT => tracing::debug_span!("-> PUT", uri),
                        Method::PATCH => tracing::debug_span!("-> PATCH", uri),
                        Method::DELETE => tracing::debug_span!("-> DELETE", uri),
                        _ => tracing::debug_span!("->", method, uri),
                    }
                }),
        );

        #[cfg(debug_assertions)]
        let host_services =
            host_services.layer(LiveReloadLayer::new().request_predicate(not_htmx_predicate));
        #[cfg(not(debug_assertions))]
        let host_services = host_services.layer(CompressionLayer::new());
        #[cfg(not(debug_assertions))]
        let host_services =
            host_services.layer(RequestBodyLimitLayer::new(opts.request_body_limit));

        SessionRow::init_table();
        let mut session_layer = SessionManagerLayer::new(DB.clone())
            .with_secure(true)
            .with_same_site(tower_sessions::cookie::SameSite::Strict)
            .with_expiry(Expiry::OnInactivity(time::Duration::days(7)));
        if let Ok(domain) = env::var("DOMAIN") {
            session_layer = session_layer.with_domain(domain)
        }
        let host_services = host_services.layer(session_layer);

        if opts.embed_default_assets {
            self = self.embed(DefaultAssets)
        }

        self = self.layer(host_services);

        Runtime::new()
            .unwrap()
            .block_on(async move {
                let listener = TcpListener::bind(opts.addr).await.unwrap();
                info!("Starting serving at {}", opts.addr);
                axum::serve(listener, self).await
            })
            .unwrap();
    }
}

fn init_tracing(level_filter: LevelFilter) {
    let _env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env()
        .unwrap()
        .add_directive("sqlparser::parser=info".parse().unwrap())
        .add_directive("tower_sessions_core=info".parse().unwrap())
        .add_directive("sled=info".parse().unwrap());
    let fmt_layer = tracing_subscriber::fmt::layer();
    #[cfg(debug_assertions)]
    let fmt_layer = fmt_layer
        .with_timer(tracing_subscriber::fmt::time::ChronoUtc::new(
            "%k:%M:%S%.3f".to_owned(),
        ))
        .with_filter(_env_filter);

    let _ = tracing_subscriber::registry()
        .with(level_filter)
        .with(fmt_layer)
        .try_init();
}

fn internal_req(request: &Request) -> bool {
    if request.uri().path().contains("tower-livereload") {
        true
    } else {
        false
    }
}
