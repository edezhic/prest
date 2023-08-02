use anyhow::Result;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<()> {
    lib::config::setup_env()?;
    lib::Storage::migrate()?;
    let svc = lib::service().into_make_service();

    let http_addr = SocketAddr::from(([0, 0, 0, 0], 80));
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

    Ok(())
}

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

use anyhow::Result;

pub static ENV_HOSTNAME: &str = "HOSTNAME";
pub static ENV_ORIGIN: &str = "ORIGIN";
pub static ENV_TLS_CERT_PATH: &str = "TLS_CERT_PATH";
pub static ENV_TLS_KEY_PATH: &str = "TLS_KEY_PATH";

static DEFAULT_HOSTNAME: &str = "localhost";
static DEFAULT_TLS_CERT_PATH: &str = "./cert.pem";
static DEFAULT_TLS_KEY_PATH: &str = "./key.pem";

pub fn setup_env() -> Result<()> {
    use std::env::{set_var, var};
    // try to read .env variables into std::env::var
    dotenv::dotenv().ok();

    let hostname = var(ENV_HOSTNAME).unwrap_or(DEFAULT_HOSTNAME.to_owned());
    let origin = format!("https://{hostname}");
    set_var(ENV_ORIGIN, origin);

    use std::{fs::write, path::Path};
    if var(ENV_TLS_CERT_PATH).is_err() || var(ENV_TLS_KEY_PATH).is_err() {
        if !Path::new(DEFAULT_TLS_CERT_PATH).exists() || !Path::new(DEFAULT_TLS_KEY_PATH).exists() {
            let cert = rcgen::generate_simple_self_signed(vec![hostname])?;
            write(DEFAULT_TLS_CERT_PATH, cert.serialize_pem()?)?;
            write(DEFAULT_TLS_KEY_PATH, cert.serialize_private_key_pem())?;
        }
        set_var(ENV_TLS_CERT_PATH, DEFAULT_TLS_CERT_PATH.to_owned());
        set_var(ENV_TLS_KEY_PATH, DEFAULT_TLS_KEY_PATH.to_owned());
    }
    Ok(())
}

