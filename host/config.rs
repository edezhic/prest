use anyhow::{Result, bail};
use tracing_subscriber::{
    filter::LevelFilter, fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
    Layer,
};
use std::{path::Path, fs::write};

pub static ENV_HOSTNAME: &str = "HOSTNAME";
pub static ENV_ORIGIN: &str = "ORIGIN";
pub static ENV_TLS_CERT_PATH: &str = "TLS_CERT_PATH";
pub static ENV_TLS_KEY_PATH: &str = "TLS_KEY_PATH";
pub static ENV_GOOGLE_CLIENT_ID: &str = "GOOGLE_CLIENT_ID";
pub static ENV_GOOGLE_CLIENT_SECRET: &str = "GOOGLE_CLIENT_SECRET";

static DEFAULT_HOSTNAME: &str = "localhost";
static DEFAULT_TLS_CERT_PATH: &str = "./cert.pem";
static DEFAULT_TLS_KEY_PATH: &str = "./key.pem";


pub fn config_env() -> Result<()> {
    use std::env::{var, set_var};
    // try to read .env variables into std::env::var
    dotenv::dotenv().ok();
    
    let hostname = var(ENV_HOSTNAME).unwrap_or(DEFAULT_HOSTNAME.to_owned());
    #[cfg(feature = "https")]
    let origin = format!("https://{hostname}");
    #[cfg(not(feature = "https"))]
    let origin = format!("http://{hostname}");
    set_var(ENV_ORIGIN, origin);

    #[cfg(feature = "https")]
    {
        if var(ENV_TLS_CERT_PATH).is_err() || var(ENV_TLS_KEY_PATH).is_err() {
            if !Path::new(DEFAULT_TLS_CERT_PATH).exists() || !Path::new(DEFAULT_TLS_KEY_PATH).exists() {
                let cert = rcgen::generate_simple_self_signed(vec![hostname])?;
                write(DEFAULT_TLS_CERT_PATH, cert.serialize_pem()?)?;
                write(DEFAULT_TLS_KEY_PATH, cert.serialize_private_key_pem())?;
            }
            set_var(ENV_TLS_CERT_PATH, DEFAULT_TLS_CERT_PATH.to_owned());
            set_var(ENV_TLS_KEY_PATH, DEFAULT_TLS_KEY_PATH.to_owned());
        }
    }

    #[cfg(feature = "oauth")]
    if var(ENV_GOOGLE_CLIENT_ID).is_err() || var(ENV_GOOGLE_CLIENT_SECRET).is_err() {
        bail!("Missing Google OAuth credentials!")
    }

    // initialize log printing
    #[cfg(debug_assertions)]
    let fmt_layer = fmt::Layer::default().with_filter(LevelFilter::DEBUG);
    #[cfg(not(debug_assertions))]
    let fmt_layer = fmt::Layer::default().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(fmt_layer).init();
    
    Ok(())
}
