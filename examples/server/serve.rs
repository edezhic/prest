use prest::*;
fn main() {
    let router = Router::new().route("/", get("Hello world!"));
    serve(router, Default::default())
}
