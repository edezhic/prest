#[tokio::main]
async fn main() {
    let service = prest::Router::new().route("/", prest::get(|| async { "Hello world!" }));
    prest::host::serve(service, Default::default()).await.unwrap();
}
