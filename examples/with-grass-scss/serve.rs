use prest::*;

include_build_output_as!(Dist);

#[tokio::main]
async fn main() {
    let service = Router::new()
        .route("/", get(html!{(Head::default().css("/styles.css")) h1{"Hello SASS!"}}))
        .embed(Dist);
    serve(service, Default::default()).await
}
