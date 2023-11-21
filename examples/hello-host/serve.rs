use prest::*;
fn main() {
    Router::new()
        .route("/", get("Hello world!"))
        .serve(ServeOptions::default())
}
