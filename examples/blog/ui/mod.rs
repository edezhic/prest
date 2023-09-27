mod templates;
pub use templates::*;
use prest::*;

macro_rules! render {
    ($template: ident) => {
        prest::get(|| async {
            (
                [(prest::header::CONTENT_TYPE, "text/html; charset=utf-8")],
                $template::render().0,
            )
        })
    };
}

pub fn service() -> Router {
    Router::new()
        .route("/", render!(home))
        .route("/platforms", render!(platforms))
        .route("/motivations", render!(motivations))
        .route("/internals", render!(internals))
        .layer(Htmxify::wrap(full_html))
}

pub fn full_html(content: Markup) -> Markup {
    maud::html!(
        html {
            (prest::head("Prest Blog", Some(maud::html!(
                link rel="stylesheet" href="/ui.css" {}
                link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Roboto:wght@300;400;500;700&display=swap"{}
                script src="/ui.js"{}
                script src="https://unpkg.com/htmx.org@1.9.0" integrity="sha384-aOxz9UdWG0yBiyrTwPeMibmaoq07/d3a96GCbb9x60f3mOt5zwkjdbcHFnKH8qls" crossorigin="anonymous"{}
            ))))
            body {(content)}
        }
    )
}
