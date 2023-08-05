use super::*;

mod embed;
pub use embed::embed;
pub mod auth;
pub use axum_server::tls_rustls::RustlsConfig;

pub fn set_dot_env_variables() {
    dotenv::dotenv().unwrap();
}

use rand::{distributions::Standard, prelude::Distribution};
pub fn generate_secret<T>() -> T 
    where Standard: Distribution<T>
{
    rand::Rng::gen::<T>(&mut rand::thread_rng())
}

pub async fn serve(router: Router, port: u16) -> Result<()> {
    let svc = router.into_make_service();
    let http_addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    axum_server::bind(http_addr).serve(svc).await?;
    anyhow::bail!("Server stopped without emitting errors")
}

pub async fn serve_tls(router: Router, tls_config: RustlsConfig) -> Result<()> {
    let svc = router.into_make_service();
    let https_addr = std::net::SocketAddr::from(([127, 0, 0, 1], 443));
    axum_server::bind_rustls(https_addr, tls_config)
        .serve(svc)
        .await?;
    anyhow::bail!("Server stopped without emitting errors")
}

pub async fn redirect_to_origin<N: AsRef<str>>(origin: N) -> Result<()> {
    use axum::handler::HandlerWithoutStateExt;
    use std::net::SocketAddr;
    let origin = origin.as_ref().to_owned();
    let redirect = |uri: http::Uri| async move {
        let path = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");
        let target = format!("{origin}{path}");
        Redirect::permanent(&target)
    };
    axum_server::bind(SocketAddr::from(([127, 0, 0, 1], 80)))
        .serve(redirect.into_make_service())
        .await?;
    Ok(())
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

