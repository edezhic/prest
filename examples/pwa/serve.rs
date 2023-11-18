use prest::*;

include_build_output_as!(Dist);

fn main() {
    shared::routes().embed(Dist).serve(Default::default())
}
