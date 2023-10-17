use prest::*;

#[derive(rust_embed::RustEmbed, Clone)]
#[folder = "$OUT_DIR/assets"]
struct Assets;

#[tokio::main]
async fn main() {
    let service = Router::new()
        .route("/", template!{(Head::default().js("/script.js")) h1{"Hello TypeScript!"}})
        .layer(embed(Assets));
    serve(service, Default::default()).await.unwrap();
}
