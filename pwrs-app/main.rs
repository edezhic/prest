#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // start logging
    use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*, Layer};
    let fmt_layer = fmt::Layer::default().with_filter(LevelFilter::DEBUG);
    tracing_subscriber::registry().with(fmt_layer).init();
    // initialize host server
    let svc = lib::service().into_make_service();
    let http_addr = std::net::SocketAddr::from(([0, 0, 0, 0], 80));
    axum_server::bind(http_addr).serve(svc).await?;
    Ok(())
}
