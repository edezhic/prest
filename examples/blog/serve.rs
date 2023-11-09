use prest::*;

include_build_output_as!(Dist);

fn main() {
    let host_routes = shared::routes()
        .embed(Dist)
        .route("/styles.css", get(Css(include_str!("assets/styles.css"))))
        .route(
            "/favicon.ico",
            get(Favicon(include_bytes!("assets/favicon.ico").as_slice())),
        );
    serve(host_routes, Default::default())
}
