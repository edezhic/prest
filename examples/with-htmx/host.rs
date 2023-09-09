#[derive(rust_embed::RustEmbed, Clone)]
#[folder = "./pub"]
struct Assets;

#[tokio::main]
async fn main() {
    let service = prest::Router::new()
        .merge(shared::ui::service())
        .layer(prest::host::embed(Assets));
    prest::host::serve(service, 80).await.unwrap();
}

