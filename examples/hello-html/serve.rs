use prest::*;
fn main() {
    Router::new()
        .route("/", get(html! {(Head::example()) h1{"Hello world!"}}))
        .serve(ServeOptions::default())
}
