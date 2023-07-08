use anyhow::Result;
use tokio::{net::TcpListener, runtime::Builder};
use tracing::{error, trace};
use tracing_subscriber::{
    filter::LevelFilter, fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt,
    Layer,
};


fn main() -> Result<()>{
    // try to read .env variables into std::env::var
    match dotenv::dotenv() {
        Ok(_) => {},
        Err(_) => {
            // set default env values
            std::env::set_var("PORT", "80");
        } 
    }
    let port = std::env::var("PORT")?.parse::<u16>()?;
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    // initialize log printing
    #[cfg(debug_assertions)]
    let fmt_layer = fmt::Layer::default().with_filter(LevelFilter::DEBUG);
    #[cfg(not(debug_assertions))]
    let fmt_layer = fmt::Layer::default().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(fmt_layer).init();
    
    // build an asynchronous runtime with all available features and start it
    let runtime = Builder::new_multi_thread().enable_all().build()?;
    runtime.block_on(async {
        // initialize the host service that will be used to process requests
        let svc = host::service().await;
        // connect to the socket
        let listener = TcpListener::bind(addr).await.unwrap();
        trace!("started listening on {}", addr);
        loop {
            let (stream, remote_addr) = listener.accept().await.unwrap();
            trace!("receiving stream from {:?}", remote_addr);
            let service = svc.clone();
            // move stream and svc clone into a separate future to process it concurrently
            tokio::task::spawn(async move {
                if let Err(err) = hyper::server::conn::Http::new()
                    .serve_connection(stream, service)
                    .with_upgrades()
                    .await
                {
                    error!(
                        "processing stream from {:?} resulted in an error: {:?}",
                        remote_addr, err
                    );
                } else {
                    trace!("gracefully closing the connection with {:?}", remote_addr);
                }
            });
        }
    });
    Ok(())
}
