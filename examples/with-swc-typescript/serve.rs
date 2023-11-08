use prest::*;

include_build_output_as!(Dist);

fn main() {
    let router = Router::new()
        .route("/", get(
            html!{h1{"Hello TypeScript!"} Scripts::empty().include("/script.js")}
        ))
        .embed(Dist);
    serve(router, Default::default())
}
