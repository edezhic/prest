use prest::*;

embed_build_output_as!(Dist);

fn main() {
    route(
        "/",
        get(html! {h1{"Hello TypeScript!"} (Scripts::default().include("/script.js"))}),
    )
    .embed(Dist)
    .run()
}
