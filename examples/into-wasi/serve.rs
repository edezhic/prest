use prest::*;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let service = Router::new().route("/", get(html!((Head::default()) h1{"Hello world!"})));
    serve(service, Default::default()).await
}
