use prest::*;

static STATE: Lazy<String> = Lazy::new(|| "value".to_owned());

embed_as!(ExampleFiles from ".");

fn main() {
    Router::new()
        .route("/", get("Hello world!"))
        .route(
            "/closure",
            get(|Host(host)| async move { format!("host: {host}, state: {}", *STATE) }),
        )
        .route("/fn", get(handler))
        .embed(ExampleFiles)
        .layer(from_fn(middleware))
        .serve(ServeOptions::default())
}

async fn handler(Host(host): Host) -> impl IntoResponse {
    format!("host: {host}, state: {}", *STATE)
}

async fn middleware(req: Request, next: Next) -> impl IntoResponse {
    let req_uri = req.uri().to_string();
    let mut response = next.run(req).await;
    response
        .headers_mut()
        .append("LAYER_HEADER", req_uri.parse().unwrap());
    response
}
