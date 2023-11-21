use prest::*;

embed_build_output_as!(Dist);

fn main() {
    shared::routes().embed(Dist).serve(ServeOptions::default())
}
