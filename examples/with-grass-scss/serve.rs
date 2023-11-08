use prest::*;

include_build_output_as!(Dist);

fn main() {
    let router = Router::new()
        .route("/", get(html!{(Head::default().css("/styles.css")) h1{"Hello SASS!"}}))
        .embed(Dist);
    serve(router, Default::default())
}
