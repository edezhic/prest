#![feature(allocator_api)]

mod embed;
pub use embed::*;

pub async fn serve(router: pwrs::Router, port: u16) -> anyhow::Result<()> {
    let svc = router.into_make_service();
    let http_addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    axum_server::bind(http_addr).serve(svc).await?;
    anyhow::bail!("Server stopped without errors")
}

pub fn init_logging() {
    use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*, Layer};
    let fmt_layer = fmt::Layer::default().with_filter(LevelFilter::DEBUG);
    tracing_subscriber::registry().with(fmt_layer).init();
}

use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    trace::TraceLayer,
};
pub fn http_tracing() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
}

