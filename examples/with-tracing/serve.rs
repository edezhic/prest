use prest::*;

#[tokio::main]
async fn main() {
    start_printing_traces();
    let svc = Router::new()
        .route(
            "/",
            get(html!(
                (Head::default())
                body { h1{"With tracing (check out the terminal!)"}}
            )),
        )
        .layer(http_tracing());
    serve(svc, Default::default()).await
}
