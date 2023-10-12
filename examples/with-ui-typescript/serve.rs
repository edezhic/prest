#![allow(dead_code)]
use prest::*;

fn routes() -> Router {
    Router::new().route("/",template!((Head::default().with(html!(script src="/script.js"{}))) body {"Hello TypeScript!"}))
}

#[derive(rust_embed::RustEmbed, Clone)]
#[folder = "$OUT_DIR/assets"]
struct Assets;

#[tokio::main]
async fn main() {
    let service = routes().layer(embed(Assets));
    serve(service, Default::default()).await.unwrap();
}
