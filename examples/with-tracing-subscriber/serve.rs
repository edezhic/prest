use prest::*;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*, Layer};

fn main() {
    let fmt_layer = fmt::Layer::default().with_filter(LevelFilter::TRACE);
    tracing_subscriber::registry().with(fmt_layer).init();

    Router::new()
        .route(
            "/",
            get(html!(
                (Head::example())
                body { h1{"With tracing (check out the terminal!)"}}
            )),
        )
        .layer(TraceLayer::new_for_http())
        .serve(Default::default())
}
