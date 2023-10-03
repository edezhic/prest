use crate::*;

pub struct Addr {
    pub ip: [u8; 4],
    pub port: u16
}

impl Default for Addr {
    fn default() -> Self {
        Self {
            ip: [0, 0, 0, 0],
            port: 80,
        }
    }
}

pub async fn serve(router: Router, addr: Addr) -> Result<()> {
    let svc = router.into_make_service();
    let socket_addr = std::net::SocketAddr::from((addr.ip, addr.port));
    axum_server::bind(socket_addr).serve(svc).await?;
    anyhow::bail!("Server stopped without emitting errors")
}

#[cfg(feature = "tls")]
pub use axum_server::tls_rustls::RustlsConfig;
#[cfg(feature = "tls")]
pub async fn serve_tls(router: Router, tls_config: RustlsConfig) -> Result<()> {
    let svc = router.into_make_service();
    let https_addr = std::net::SocketAddr::from(([127, 0, 0, 1], 443));
    axum_server::bind_rustls(https_addr, tls_config)
        .serve(svc)
        .await?;
    anyhow::bail!("Server stopped without emitting errors")
}
#[cfg(feature = "tls")]
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