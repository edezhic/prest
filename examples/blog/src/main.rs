use prest::*;

embed_build_output_as!(BuiltAssets);

fn main() {
    init!();
    blog::routes().embed(BuiltAssets).run()
}
