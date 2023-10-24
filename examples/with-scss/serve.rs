use prest::*;

embed!(Dist);

#[tokio::main]
async fn main() {
    let service = Router::new()
        .route("/", get(html!{(Head::default().css("/dist/styles.css")) h1{"Hello SASS!"}}))
        .merge(Dist::routes("/dist/"));
    serve(service, Default::default()).await
}
