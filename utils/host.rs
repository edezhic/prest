use crate::*;

pub async fn serve(router: Router, opts: Addr) {
    let svc = router.into_make_service();
    let socket_addr = SocketAddr::from((opts.ip, opts.port));
    hyper_server::bind(socket_addr).serve(svc).await.unwrap();
}

#[cfg(feature = "tls")]
pub use hyper_server::tls_rustls::RustlsConfig;
#[cfg(feature = "tls")]
pub async fn serve_tls(router: Router, tls_config: RustlsConfig) {
    let svc = router.into_make_service();
    let https_addr = SocketAddr::from(([127, 0, 0, 1], 443));
    hyper_server::bind_rustls(https_addr, tls_config)
        .serve(svc)
        .await
        .unwrap();
}

#[cfg(feature = "tls")]
pub async fn redirect_to_origin<N: AsRef<str>>(origin: N) {
    use axum::handler::HandlerWithoutStateExt;
    let origin = origin.as_ref().to_owned();
    let redirect = |uri: http::Uri| async move {
        let path = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");
        let target = format!("{origin}{path}");
        Redirect::permanent(&target)
    };
    hyper_server::bind(SocketAddr::from(([127, 0, 0, 1], 80)))
        .serve(redirect.into_make_service())
        .await.unwrap();
}