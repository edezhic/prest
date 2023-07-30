use anyhow::Result;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<()> {
    lib::config::setup_env()?;
    lib::Storage::migrate()?;
    let svc = lib::service().into_make_service();

    let http_addr = SocketAddr::from(([0, 0, 0, 0], 80));
    #[cfg(feature = "https")]
    {
        tokio::spawn(redirect_to_https(http_addr));
        use axum_server::tls_rustls::RustlsConfig;
        use lib::config::{ENV_TLS_CERT_PATH, ENV_TLS_KEY_PATH};
        use std::env::var;
        let tls_config =
            RustlsConfig::from_pem_file(var(ENV_TLS_CERT_PATH)?, var(ENV_TLS_KEY_PATH)?).await?;
        let https_addr = SocketAddr::from(([0, 0, 0, 0], 443));
        axum_server::bind_rustls(https_addr, tls_config)
            .serve(svc)
            .await?;
    }
    #[cfg(not(feature = "https"))]
    axum_server::bind(http_addr).serve(svc).await?;

    Ok(())
}

#[cfg(feature = "https")]
async fn redirect_to_https(http_addr: SocketAddr) -> Result<()> {
    use axum::{handler::HandlerWithoutStateExt, response::Redirect};
    let origin = std::env::var("ORIGIN")?;

    let redirect = |uri: http::Uri| async move {
        let path = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");
        let target = format!("{origin}{path}");
        Redirect::permanent(&target)
    };

    axum_server::bind(http_addr)
        .serve(redirect.into_make_service())
        .await?;

    Ok(())
}
