use crate::*;
use axum::routing::MethodRouter;
pub use maud::*;

pub fn template(s: String) -> MethodRouter {
    get(|| async {
        Html(s)
    })
}

pub fn maud_to_response(markup: Markup) -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], markup.0)
}

pub fn maud_pwa_head(title: &str, other: Option<Markup>) -> Markup {
    maud::html!(
        head {
            title {(title)}
            link rel="icon" href="/favicon.ico" {}
            link rel="manifest" href="/.webmanifest" {}
            script {(REGISTER_SW_SNIPPET)}
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            meta name="theme-color" content="#a21caf";
            @if let Some(markup) = other {
                (markup)
            }
        }
    )
}