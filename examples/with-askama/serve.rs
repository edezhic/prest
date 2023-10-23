use askama::Template;
use prest::*;

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

#[tokio::main]
async fn main() {
    let service = Router::new().route(
        "/",
        get(Html(HelloTemplate { name: "world" }.render().unwrap())),
    );
    serve(service, Default::default()).await
}
