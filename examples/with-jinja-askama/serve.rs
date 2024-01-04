use askama::Template;
use prest::*;

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

fn main() {
    route(
        "/",
        get(Html(HelloTemplate { name: "world" }.render().unwrap())),
    )
    .run()
}
