use prest::*;

embed!(Dist);

#[tokio::main]
async fn main() {
    let service = Router::new()
        .route("/", get(html!{(Head::default().js("/dist/script.js")) h1{"Hello TypeScript!"}}))
        .route("/dist/*any", get(Dist::handle));
    serve(service, Default::default()).await
}
