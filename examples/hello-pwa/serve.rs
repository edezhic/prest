use prest::*;

embed_build_output_as!(BuiltAssets);

fn main() {
    shared::routes()
        .embed(BuiltAssets)
        .serve(ServeOptions::default())
}
