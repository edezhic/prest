mod templates;
use axum::{body::HttpBody, middleware::from_fn, routing::get, Router};
pub use templates::*;
pub use maud::{html, Markup, PreEscaped};

pub fn service() -> Router {
    Router::new()
        .route("/", get(|| async { home::render().0 }))
        .layer(from_fn(layout_wrapper))
        .layer(from_fn(add_content_type))
}

async fn layout_wrapper(
    request: http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> impl axum::response::IntoResponse {
    let is_htmx_request = request.headers().get("HX-Request").is_some();
    let response = next.run(request).await;
    let (mut parts, mut content) = response.into_parts();
    let mut buf = Vec::with_capacity(content.size_hint().lower() as usize);
    while let Some(chunk) = content.data().await {
        bytes::BufMut::put(&mut buf, chunk.unwrap());
    }
    let content = std::string::String::from_utf8(buf).unwrap();
    let content = if is_htmx_request {
        PreEscaped(content)
    } else {
        //full_html(layout::render(PreEscaped(content)))
        full_html(PreEscaped(content))
    };
    parts.headers.remove(http::header::CONTENT_LENGTH);
    axum::response::Response::from_parts(parts, content.0)
}

pub fn full_html(content: Markup) -> Markup {
    html!(
        html {
            head {
                title {"PWRS app"}
                link rel="icon" href="/favicon.png" {}
                link rel="manifest" href="/.webmanifest" {}
                link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Roboto:wght@300;400;500;700&display=swap" {}
                link rel="stylesheet" href="/ui.css" {}
                script src="/ui.js" {}
                script src="https://unpkg.com/htmx.org@1.9.0" integrity="sha384-aOxz9UdWG0yBiyrTwPeMibmaoq07/d3a96GCbb9x60f3mOt5zwkjdbcHFnKH8qls" crossorigin="anonymous" {}
                meta
                    name="viewport"
                    content="width=device-width, initial-scale=1.0";
            }
            body {(content)}
        }
    )
}

async fn add_content_type(
    request: http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> impl axum::response::IntoResponse {
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        http::header::CONTENT_TYPE,
        "text/html; charset=UTF-8".parse().unwrap(),
    );
    response
}