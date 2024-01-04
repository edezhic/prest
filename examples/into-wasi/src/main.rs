#[cfg(target_arch = "wasm32")]
fn main() {
    use hyper::server::conn::Http;
    use prest::*;
    use tokio::{net::TcpListener, runtime::Builder, task};

    let router: Router<()> =
        route("/", get(html!((Head::example("WASI")) h1{"Hello world!"})));

    let rt = Builder::new_current_thread().enable_all().build().unwrap();

    rt.block_on(async {
        let listener = TcpListener::bind(ServeOptions::default().addr)
            .await
            .unwrap();
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let svc = router.clone();
            task::spawn(async move {
                if let Err(err) = Http::new().serve_connection(stream, svc).await {
                    println!("Error serving connection: {:?}", err);
                }
            });
        }
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    panic!("This example expects wasm32-wasi compilation target!");
}
