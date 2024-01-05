use prest::*;

embed_build_output_as!(Dist);

fn main() {
    route(
        "/",
        get(html! {(Head::with_title("With SCSS").css("/styles.css")) h1{"Hello SASS!"}}),
    )
    .embed(Dist)
    .run()
}
