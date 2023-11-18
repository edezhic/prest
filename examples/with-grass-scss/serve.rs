use prest::*;

include_build_output_as!(Dist);

fn main() {
    Router::new()
        .route("/", get(html!{(Head::example().css("/styles.css")) h1{"Hello SASS!"}}))
        .embed(Dist)
        .serve(Default::default())
}
