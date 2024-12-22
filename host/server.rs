use crate::*;

use axum::handler::HandlerWithoutStateExt;
use axum_server::Handle;
use http::uri::Authority;
use rustls_acme::{caches::DirCache, AcmeConfig};
use std::net::{Ipv6Addr, SocketAddr};

pub async fn start(router: Router) -> Result<(), Error> {
    let AppConfig {
        name,
        domain,
        data_dir,
        ..
    } = APP_CONFIG.check();

    let handle = RT.new_server_handle();

    if *IS_REMOTE && domain.is_some() {
        let domain = domain.as_ref().expect("Already validated is_some");
        let mut certs_path = data_dir.clone();
        certs_path.push("certs");

        let mut state = AcmeConfig::new(vec![domain.clone()])
            .cache_option(Some(DirCache::new(certs_path)))
            .directory_lets_encrypt(true)
            .state();
        let acceptor = state.axum_acceptor(state.default_rustls_config());

        tokio::spawn(async move {
            loop {
                match state.next().await {
                    Some(Ok(ok)) => trace!(target: "server", "TLS acme event: {:?}", ok),
                    Some(Err(err)) => error!(target: "server", "TLS acme error: {:?}", err),
                    None => tokio::time::sleep(std::time::Duration::from_millis(100)).await,
                }
            }
        });

        let redirect_handle = RT.new_server_handle();
        tokio::spawn(redirect_http_to_https(redirect_handle));

        info!(target: "server", "Starting serving {name} at https://{domain}");
        axum_server::bind(SocketAddr::from((Ipv6Addr::UNSPECIFIED, 443)))
            .acceptor(acceptor)
            .handle(handle)
            .serve(router.into_make_service_with_connect_info::<SocketAddr>())
            .await?;
    } else {
        #[cfg(debug_assertions)]
        info!(target: "server", "Starting serving {name} at http://localhost");

        axum_server::bind(SocketAddr::from((Ipv6Addr::UNSPECIFIED, check_port())))
            .handle(handle)
            .serve(router.into_make_service_with_connect_info::<SocketAddr>())
            .await?;
    }
    OK
}

async fn redirect_http_to_https(handle: Handle) {
    fn make_https(host: &str, uri: Uri, https_port: u16) -> Result<Uri, BoxError> {
        let mut parts = uri.into_parts();

        parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().expect("Valid root path"));
        }

        let authority: Authority = host.parse()?;
        let bare_host = match authority.port() {
            Some(port_struct) => authority
                .as_str()
                .strip_suffix(port_struct.as_str())
                .map(|a| a.strip_suffix(':'))
                .flatten()
                .expect("Authority.port() is Some(port) then we can be sure authority ends with :{port}"),
            None => authority.as_str(),
        };

        parts.authority = Some(format!("{bare_host}:{https_port}").parse()?);

        Ok(Uri::from_parts(parts)?)
    }

    let redirect = move |Host(host): Host, uri: Uri| async move {
        match make_https(&host, uri, 443) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(error) => {
                tracing::warn!(target: "https redirect", %error, "failed to convert URI to HTTPS");
                Err(StatusCode::BAD_REQUEST)
            }
        }
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], 80));

    axum_server::bind(addr)
        .handle(handle)
        .serve(redirect.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .expect("HTTP -> HTTPS redirection service should start and end gracefully");
}

fn check_port() -> u16 {
    if let Ok(v) = env::var("PORT") {
        v.parse::<u16>().unwrap_or(80)
    } else {
        80
    }
}
