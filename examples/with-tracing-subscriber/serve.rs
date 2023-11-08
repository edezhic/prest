use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*, Layer};
use tower_http::trace::TraceLayer;
use prest::*;

fn main() {
    let fmt_layer = fmt::Layer::default().with_filter(LevelFilter::TRACE);
    tracing_subscriber::registry().with(fmt_layer).init();

    let svc = Router::new()
        .route(
            "/",
            get(html!(
                (Head::default())
                body { h1{"With tracing (check out the terminal!)"}}
            )),
        )
        .layer(TraceLayer::new_for_http());
    serve(svc, Default::default())
}
