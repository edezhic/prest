use prest::*;

// dommy lazily initialized global variable
static STATE: Lazy<String> = Lazy::new(|| "value".to_owned());

// utility macro that embeds files from path into a struct
embed_as!(ExampleFiles from ".");

fn main() {
    // initializing a new router
    Router::new()
        // assigning a handler to the GET method of a route
        .route("/", get(handler))
        // adding a middleware that affects routes above
        .layer(from_fn(middleware))
        // adding routes&handlers for the embedded files
        .embed(ExampleFiles)
        // connecting to a socket and starting processing requests
        .serve(ServeOptions::default())
}

// can accept any axum Extractors as arguments
async fn handler(Host(host): Host) -> impl IntoResponse {
    format!("host: {host}, state: {}", *STATE)
}

// itermediate processing of requests/responses
async fn middleware(req: Request, next: Next) -> impl IntoResponse {
    let req_uri = req.uri().to_string();
    let mut response = next.run(req).await;
    response
        .headers_mut()
        .append("MIDDLEWARE_HEADER", req_uri.parse().unwrap());
    response
}
