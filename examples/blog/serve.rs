use prest::*;

embed_build_output_as!(BuiltAssets);
embed_as!(StaticAssets from "assets" except "*.svg");

fn main() {
    shared::routes()
        .embed(BuiltAssets)
        .embed(StaticAssets)
        .run()
}
