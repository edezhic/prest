use askama::Template;
use prest::*;

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

fn main() {
    let router = Router::new().route(
        "/",
        get(Html(HelloTemplate { name: "world" }.render().unwrap())),
    );
    serve(router, Default::default())
}
