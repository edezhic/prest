use anyhow::Result;
use axum::response::Redirect;
use axum_server::tls_rustls::RustlsConfig;
use host::{ENV_TLS_CERT_PATH, ENV_TLS_KEY_PATH};
use http::Uri;
use std::{env::var, net::SocketAddr};

#[tokio::main]
async fn main() -> Result<()> {
    host::config_env()?;
    host::Storage::migrate()?;
    let svc = host::service().into_make_service();
    let http_addr = SocketAddr::from(([0, 0, 0, 0], 80));

    #[cfg(feature = "https")]
    {
        let tls_config =
            RustlsConfig::from_pem_file(var(ENV_TLS_CERT_PATH)?, var(ENV_TLS_KEY_PATH)?).await?;

        tokio::spawn(redirect_to_https(http_addr));
        let https_addr = SocketAddr::from(([0, 0, 0, 0], 443));
        axum_server::bind_rustls(https_addr, tls_config)
            .serve(svc)
            .await?;
    }
    #[cfg(not(feature = "https"))]
    axum_server::bind(http_addr).serve(svc).await?;

    Ok(())
}

async fn redirect_to_https(http_addr: SocketAddr) -> Result<()> {
    use axum::handler::HandlerWithoutStateExt;
    let origin = var("ORIGIN")?;

    let redirect = |uri: Uri| async move {
        let path = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");
        let target = format!("{origin}{path}");
        Redirect::permanent(&target)
    };

    axum_server::bind(http_addr)
        .serve(redirect.into_make_service())
        .await?;

    Ok(())
}
