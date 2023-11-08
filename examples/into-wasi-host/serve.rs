use prest::*;

fn main() {
    let router = Router::new().route("/", get(html!((Head::default()) h1{"Hello world!"})));
    serve(router, Default::default())
}
