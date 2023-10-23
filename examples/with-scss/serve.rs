use prest::*;

embed!(Dist);

#[tokio::main]
async fn main() {
    let service = Router::new()
        .route("/", get(html!{(Head::default().css("/dist/styles.css")) h1{"Hello SASS!"}}))
        .route("/dist/*any", get(Dist::handle));
    serve(service, Default::default()).await
}
