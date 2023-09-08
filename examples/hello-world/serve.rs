#[tokio::main]
async fn main() {
    let service = pwrs::Router::new().route("/", pwrs::get(|| async { "Hello world!" }));
    pwrs::host::serve(service, 80).await.unwrap();
}
