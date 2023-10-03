mod generator;
use prest::*;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut generator = generator::Mistral::new().unwrap();
    generator.sample("To make cross-platform app in Rust", 5).unwrap();

    let service = Router::new().route("/", get(|| async { "With Mistral inference!" }));
    host::serve(service, Default::default()).await.unwrap();
}

