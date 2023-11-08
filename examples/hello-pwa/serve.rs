use prest::*;

fn main() {
    include_build_output_as!(Dist);
    serve(shared::routes().embed(Dist), Default::default())
}
