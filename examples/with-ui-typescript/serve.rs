#![allow(dead_code)]

fn routes() -> prest::Router {
    prest::Router::new().route(
        "/",
        prest::get(|| async {
            prest::maud_to_response(
                maud::html!((prest::head("With TypeScript", Some(maud::html!(
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
    let service = routes().layer(prest::host::embed(Assets));
    prest::host::serve(service, Default::default()).await.unwrap();
}
