use super::*;

mod embed;
pub use embed::embed;
pub mod auth;

pub fn set_dot_env_variables() {
    dotenv::dotenv().unwrap();
}

use rand::{distributions::Standard, prelude::Distribution};
pub fn generate_secret<T>() -> T 
    where Standard: Distribution<T>
{
    rand::Rng::gen::<T>(&mut rand::thread_rng())
}

pub async fn serve(router: Router, port: u16) -> anyhow::Result<()> {
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

