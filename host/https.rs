use crate::*;
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;

#[allow(dead_code)]
pub async fn serve_https(router: Router) -> Result<()> {
    let tls_config = RustlsConfig::from_pem_file("./cert.pem", "./key.pem").await?;

    // init http -> https redirection service
    tokio::spawn(redirect_to_https());
    
    let https_addr = SocketAddr::from(([0, 0, 0, 0], 443));
    info!("Starting serving at {https_addr}");
    axum_server::bind_rustls(https_addr, tls_config)
        .handle(SHUTDOWN.new_server_handle())
        .serve(router.into_make_service())
        .await?;
    Ok(())
}

#[allow(dead_code)]
async fn redirect_to_https() -> Result<()> {
    use axum::handler::HandlerWithoutStateExt;
    let redirect = |host: Host, uri: http::Uri| async move {
        let path = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");
        Redirect::permanent(&format!("https://{}{path}", host.0))
    };
    let http_addr = SocketAddr::from(([0, 0, 0, 0], 80));
    info!("Starting redirecting to https at {http_addr}");
    axum_server::bind(http_addr)
        .handle(SHUTDOWN.new_server_handle())
        .serve(redirect.into_make_service())
        .await?;
    Ok(())
}
