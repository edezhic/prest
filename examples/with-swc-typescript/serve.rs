use prest::*;

#[derive(Embed)] 
struct Dist;

#[tokio::main]
async fn main() {
    let service = Router::new()
        .route("/", get(html!{(Head::default().js("/script.js")) h1{"Hello TypeScript!"}}))
        .embed::<Dist>();
    serve(service, Default::default()).await
}
