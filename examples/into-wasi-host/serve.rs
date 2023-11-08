use prest::*;

fn main() {
    let router = Router::new().route("/", get(html!((Head::example()) h1{"Hello world!"})));
    serve(router, Default::default())
}
