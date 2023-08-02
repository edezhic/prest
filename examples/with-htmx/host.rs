#[derive(rust_embed::RustEmbed, Clone)]
#[folder = "./pub"]
struct Assets;

#[tokio::main]
async fn main() {
    let service = pwrs::Router::new()
        .merge(shared::ui::service())
        .layer(pwrs_host::embed(Assets));
    pwrs_host::serve(service, 80).await.unwrap();
}

