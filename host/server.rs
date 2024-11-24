use crate::*;

use rustls_acme::{caches::DirCache, AcmeConfig};
use std::net::{Ipv6Addr, SocketAddr};

pub async fn start(router: Router) -> Result<(), Error> {
    let app = APP_CONFIG.check();
    let name = app.name.clone();
    let domain = app.domain.clone();

    let handle = SHUTDOWN.new_server_handle();

    if *IS_REMOTE {
        if let Some(domain) = domain {
            let project_dirs = prest::ProjectDirs::from("", "", &name).unwrap();
            let mut certs_path = project_dirs.data_dir().to_path_buf();
            certs_path.push("certs");

            let mut state = AcmeConfig::new(vec![domain.clone()])
                .cache_option(Some(DirCache::new(certs_path)))
                .directory_lets_encrypt(true)
                .state();
            let acceptor = state.axum_acceptor(state.default_rustls_config());

            tokio::spawn(async move {
                loop {
                    match state.next().await {
                        Some(Ok(ok)) => trace!("TLS acme event: {:?}", ok),
                        Some(Err(err)) => error!("TLS acme error: {:?}", err),
                        None => tokio::time::sleep(std::time::Duration::from_millis(100)).await,
                    }
                }
            });

            info!("Starting serving {name} at https://{domain}");
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
        info!("Starting serving {name} at http://localhost");
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
