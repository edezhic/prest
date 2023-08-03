#[derive(rust_embed::RustEmbed, Clone)]
#[folder = "./pub"]
struct Assets;

#[tokio::main]
async fn main() {
    let service = pwrs::Router::new()
        .merge(shared::service())
        .layer(pwrs::host::embed(Assets));
    pwrs::host::serve(service, 80).await.unwrap();
}

