use prest::*;

embed_build_output_as!(BuiltAssets);

fn main() {
    hello_pwa::shared().embed(BuiltAssets).run()
}
