use askama::Template;
use prest::*;

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

fn main() {
    Router::new()
        .route(
            "/",
            get(Html(HelloTemplate { name: "world" }.render().unwrap())),
        )
        .serve(ServeOptions::default())
}
