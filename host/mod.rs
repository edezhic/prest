use crate::*;

mod admin;
mod state;

mod shutdown;
use axum::middleware;
use serde::{Deserialize, Serialize};
pub use shutdown::*;

mod schedule;
pub use schedule::*;

mod config;
pub use config::*;

#[cfg(feature = "auth")]
mod auth;
#[cfg(feature = "auth")]
pub use auth::*;

#[cfg(feature = "https")]
mod https;

#[cfg(feature = "traces")]
mod traces;
#[cfg(feature = "traces")]
use traces::*;
pub use traces::init_tracing_subscriber;

#[cfg(feature = "webview")]
mod webview;

pub use axum::response::sse::{Event as SseEvent, KeepAlive as SseKeepAlive, Sse};
use std::net::SocketAddr;
pub use tokio::{
    runtime::{Builder as RuntimeBuilder, Handle as RuntimeHandle, Runtime, RuntimeFlavor},
    sync::{Mutex, OnceCell, RwLock},
    task::block_in_place,
};
pub use toml::{Table as TomlTable, Value as TomlValue};
pub use directories::*;
pub use dotenvy::dotenv;
pub type SseItem = Result<SseEvent, Infallible>;
#[cfg(feature = "db")]
pub(crate) use gluesql::sled_storage::SledStorage as PersistentStorage;

use std::collections::HashMap;
#[derive(Debug, Table, Serialize, Deserialize)]
pub struct RouteStats {
    path: String,
    hits: i64,
    statuses: HashMap<u16, u64>,
}

type Port = u16;

/// Utility trait to use Router as the host
pub trait HostUtils {
    /// Init env vars, DB, auth, tracing, other utils and start the server
    fn run(self);
    fn serve(self);
    fn add_utility_layers(self) -> Self;
    fn add_default_embeddings(self) -> Self;
    fn add_route_stats(self) -> Self;
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
            .add_route_stats()
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
    fn add_admin(self) -> Self {
        let mut router = self;
        for table in DB_SCHEMA.tables() {
            router = router.route(table.path(), get(|| async {
                let schema = table.schema();
                let values = table.get_all();
                let mut rows = vec![];
                for row_values in values {
                    let mut cells = vec![];
                    let key_selector = format!("a{}", row_values[0].clone());
                    let inputs_classname = format!(".{key_selector}");
                    
                    for (schema, value) in std::iter::zip(schema, row_values) {
                        let input_type = match schema.glue_type {
                                "BOOLEAN" => "checkbox",
                                t if t.starts_with("UINT") || t.starts_with("INT") || t.starts_with("F") => "number",
                                "U64" | "U8" | "F64" => "number",
                                "TEXT" | _ => "text",
                            
                        };
                
                        let cell_class = match schema.key {
                            true => "hidden",
                            false => "text-center",
                        };
                
                        let input_class = match input_type {
                            "text" | "number" => "input input-bordered w-full",
                            "checkbox" => "checkbox",
                            _ => "",
                        };

                        let checked = match value.as_str() {
                            "true" => true,
                            _ => false,
                        };

                        let onchange = match input_type {
                            "checkbox" => Some("this.value = this.checked ? 'true' : 'false'"),
                            _ => None
                        };

                        let cell = html! {
                            td.(cell_class) {input.(input_class).(key_selector) onchange=[(onchange)] type=(input_type) name=(schema.name) value=(value) checked[checked] {}}
                        };
                        cells.push(cell);
                    }
                    rows.push(html!(tr #(key_selector) ."relative" { 
                        @for cell in cells {(cell)}
                        td."flex justify-around items-center" {
                            button hx-post=(table.path()) hx-swap="none" hx-include=(inputs_classname) type="submit" {"Save"}
                            button hx-delete=(table.path()) hx-swap="outerHtml" hx-target=(format!("#{key_selector}")) hx-include=(inputs_classname) type="submit" {"Delete"}   
                        }
                    }))
                }
                html!(
                    table."w-full" {
                        @let headers = table.schema().iter().filter(|c| !c.key).map(|c| c.name);
                        @for header in headers {th {(header)}} th{"Actions"}
                        @for row in rows {(row)}
                    }
                )
            })).route(table.path(), post(|req: Request| async {
                table.save(req).await
            })).route(table.path(), delete(|req: Request| async {
                table.remove(req).await
            }));
        }
        router.route("/admin", get(admin::page))
    }
    fn add_route_stats(self) -> Self {
        RouteStats::migrate();
        self.layer(middleware::from_fn(|request: Request, next: Next| async {
            let path = match internal_req(&request) {
                true => None,
                false => Some(request.uri().path().to_owned()),
            };

            let response = next.run(request).await;

            if let Some(path) = path {
                let status = response.status().as_u16();
                let mut stats = RouteStats::find_by_path(&path).unwrap_or(RouteStats {
                    path,
                    hits: 0,
                    statuses: HashMap::new(),
                });
                stats.hits += 1;
                stats
                    .statuses
                    .entry(status)
                    .and_modify(|v| *v += 1)
                    .or_insert(1);
                if let Err(e) = stats.save() {
                    tracing::warn!("Failed to update stats: {e}");
                }
            }
            response
        }))
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

fn check_port() -> Port {
    if let Ok(v) = env::var("PORT") {
        v.parse::<Port>().unwrap_or(80)
    } else {
        80
    }
}

#[allow(dead_code)]
fn localhost(port: Port) -> String {
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
