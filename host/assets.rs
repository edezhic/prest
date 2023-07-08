use axum::{
    body::Body,
    extract::Request,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};

#[derive(rust_embed::RustEmbed)]
#[folder = "../pub"]
struct Assets;

pub async fn static_handler(request: Request) -> impl IntoResponse {
    let mut response = Response::builder();
    // remove leading slashes
    let path = request.uri().path().trim_start_matches('/');
    // lookup the pub folder
    match Assets::get(path) {
        Some(content) => {
            let content_etag = hex::encode(content.metadata.sha256_hash());
            if let Some(request_etag) = request.headers().get(http::header::IF_NONE_MATCH) {
                if request_etag.as_bytes() == content_etag.as_bytes() {
                    // respond with empty 304 to notify that file did not change
                    return response
                        .status(http::StatusCode::NOT_MODIFIED)
                        .body(Body::empty())
                        .unwrap();
                }
            }
            if let Some(mime) = mime_guess::from_path(path).first() {
                response = response.header(header::CONTENT_TYPE, mime.as_ref());
            }  
            response
                .header(header::ETAG, content_etag)
                .body(Body::from(content.data))
                .unwrap()
        }
        None => response
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap(),
    }
}
