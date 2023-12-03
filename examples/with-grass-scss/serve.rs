use prest::*;

embed_build_output_as!(Dist);

fn main() {
    Router::new()
        .route("/", get(html!{(Head::example("With SCSS").css("/styles.css")) h1{"Hello SASS!"}}))
        .embed(Dist)
        .serve(ServeOptions::default())
}
