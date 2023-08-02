#[derive(rust_embed::RustEmbed, Clone)]
#[folder = "./pub"]
struct Assets;

#[tokio::main]
async fn main() {
    pwrs_host::init_logging();
    let service = pwrs::Router::new()
        .merge(shared::service())
        .layer(pwrs_host::embed(Assets))
        .layer(pwrs_host::http_tracing());
    pwrs_host::serve(service, 80).await.unwrap();
}

