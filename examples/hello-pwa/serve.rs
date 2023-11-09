use prest::*;

include_build_output_as!(Dist);

fn main() {
    let host_routes = shared::routes().embed(Dist);
    serve(host_routes, Default::default())
}
