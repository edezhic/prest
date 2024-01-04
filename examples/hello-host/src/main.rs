use prest::*;

// utility macro that embeds files
embed_as!(ExampleFiles from ".");

// utility macro that enables global variables
state!(STATE: &'static str = { "value" });
// also supports async initialization
state!(ASTATE: &'static str = async { "value from async" });

fn main() {
    // initializing a new router from a GET handler of the homepage
    route("/", get(handler))
        // adding a middleware that affects routes above
        .layer(from_fn(middleware))
        // adding routes&handlers for the embedded files
        .embed(ExampleFiles)
        // connecting to a socket and starting processing requests
        .run()
}

// can accept any axum Extractors as arguments
async fn handler(Host(host): Host, session: Session) -> impl IntoResponse {
    let counter: usize = session.get("counter").await.unwrap().unwrap_or_default();
    session.insert("counter", counter + 1).await.unwrap();
    format!("count: {}, host: {host}, state: {}, async_state: {}", counter, *STATE, *ASTATE)
}

// intermediate processing of requests/responses
async fn middleware(req: Request, next: Next) -> impl IntoResponse {
    let req_uri = req.uri().to_string();
    let mut response = next.run(req).await;
    response
        .headers_mut()
        .append("MIDDLEWARE_HEADER", req_uri.parse().unwrap());
    response
}
