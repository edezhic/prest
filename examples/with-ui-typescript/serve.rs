#![allow(dead_code)]
use prest::*;

fn routes() -> Router {
    Router::new().route(
        "/",
        get(|| async {
            maud_to_response(
                maud::html!((maud_pwa_head("With TypeScript", Some(maud::html!(
                    script src="/script.js"{}
                )))) body {"Hello world with TypeScript!"}),
            )
        }),
    )
}

#[derive(rust_embed::RustEmbed, Clone)]
#[folder = "$OUT_DIR/assets"]
struct Assets;

#[tokio::main]
async fn main() {
    let service = routes().layer(embed(Assets));
    serve(service, Default::default()).await.unwrap();
}
