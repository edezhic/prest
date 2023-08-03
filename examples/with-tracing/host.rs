#[derive(rust_embed::RustEmbed, Clone)]
#[folder = "./pub"]
struct Assets;

#[tokio::main]
async fn main() {
    pwrs::host::init_logging();
    let service = pwrs::Router::new()
        .merge(shared::service())
        .layer(pwrs::host::embed(Assets))
        .layer(pwrs::host::http_tracing());
    pwrs::host::serve(service, 80).await.unwrap();
}

