use prest::*;

embed!(Dist);

#[tokio::main]
async fn main() {
    let service = Router::new()
        .route("/", get(html!{(Head::default().js("/dist/script.js")) h1{"Hello TypeScript!"}}))
        .merge(Dist::routes("/dist/"));
    serve(service, Default::default()).await
}
