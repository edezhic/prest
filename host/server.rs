use crate::*;

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

    if *IS_REMOTE {
        if let Some(domain) = domain {
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

            info!(target: "server", "Starting serving {name} at https://{domain}");
            axum_server::bind(SocketAddr::from((Ipv6Addr::UNSPECIFIED, 443)))
                .acceptor(acceptor)
                .handle(handle)
                .serve(router.into_make_service_with_connect_info::<SocketAddr>())
                .await?;
        } else {
            axum_server::bind(SocketAddr::from((Ipv6Addr::UNSPECIFIED, check_port())))
                .handle(handle)
                .serve(router.into_make_service_with_connect_info::<SocketAddr>())
                .await?;
        }
    } else {
        info!(target: "server", "Starting serving {name} at http://localhost");
        axum_server::bind(SocketAddr::from((Ipv6Addr::UNSPECIFIED, check_port())))
            .handle(handle)
            .serve(router.into_make_service_with_connect_info::<SocketAddr>())
            .await?;
    }
    OK
}

fn check_port() -> u16 {
    if let Ok(v) = env::var("PORT") {
        v.parse::<u16>().unwrap_or(80)
    } else {
        80
    }
}
