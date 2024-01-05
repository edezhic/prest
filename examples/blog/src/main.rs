use prest::*;

embed_build_output_as!(BuiltAssets);

fn main() {
    blog::routes().embed(BuiltAssets).run()
}
