use axum::{middleware::from_fn, routing::get, Router};
use maud::Markup;
use pwrs_ui::{add_html_content_type, Htmxify};

mod templates;
pub use templates::*;

pub fn service() -> Router {
    Router::new()
        .route("/", get(|| async { home::render().0 }))
        .layer(Htmxify::wrap(&(full_html::render as fn(Markup) -> Markup)))
        .layer(from_fn(add_html_content_type))
}
