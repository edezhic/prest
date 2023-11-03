use prest::*;

include_build_output_as!(Dist);

#[tokio::main]
async fn main() {
    let service = Router::new()
        .route("/", get(
            html!{h1{"Hello TypeScript!"} Scripts::empty().include("/script.js")}
        ))
        .embed(Dist);
    serve(service, Default::default()).await
}
