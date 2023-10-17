use prest::*;

#[tokio::main]
async fn main() {
    let service = Router::new().route("/", template!((Head::default()) h1{"Hello world!"}));
    serve(service, Default::default()).await.unwrap();
}
