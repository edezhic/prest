use prest::*;

#[tokio::main]
async fn main() {
    let service = Router::new().route("/", get(html!(h1{"Hello world!"})));
    serve(service, Default::default()).await
}