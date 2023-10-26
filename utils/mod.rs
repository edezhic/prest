use crate::*;
pub use std::net::SocketAddr;

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

#[cfg(feature = "sw")]
mod sw;
#[cfg(feature = "sw")]
pub use sw::*;

#[cfg(feature = "host")]
mod host;
#[cfg(feature = "host")]
pub use host::*;

#[cfg(feature = "host-wasi")]
mod host_wasi {
    use crate::*;
    use hyper::server::conn::Http;
    use tokio::net::TcpListener;
    
    pub async fn serve(router: Router, opts: Addr) {        
        let addr = SocketAddr::from((opts.ip, opts.port));
        let listener = TcpListener::bind(addr).await.unwrap();
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let svc = router.clone();
            tokio::task::spawn(async move {
                if let Err(err) = Http::new().serve_connection(stream, svc).await {
                    println!("Error serving connection: {:?}", err);
                }
            });
        }
    }    
}
#[cfg(feature = "host-wasi")]
pub use host_wasi::serve;

#[cfg(feature = "print-traces")]
pub fn start_printing_traces() {
    use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*, Layer};
    let fmt_layer = fmt::Layer::default().with_filter(LevelFilter::DEBUG);
    tracing_subscriber::registry().with(fmt_layer).init();
}

#[cfg(feature = "oauth")]
pub mod oauth;

#[cfg(feature = "dot_env")]
pub fn set_dot_env_variables() {
    dotenv::dotenv().unwrap();
}

#[cfg(feature = "random")]
pub fn generate_secret<T>() -> T 
    where rand::distributions::Standard: rand::prelude::Distribution<T>
{
    rand::Rng::gen::<T>(&mut rand::thread_rng())
}
