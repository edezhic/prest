use crate::*;
use rust_embed::RustEmbed;
use std::{task::{Context, Poll}, pin::Pin, future::Future, alloc::Global};

pub fn embed<T: RustEmbed + Clone>(assets: T) -> Embed<T> {
    Embed { assets }
}

#[derive(Clone)]
pub struct Embed<T> where T: RustEmbed + Clone {
    pub assets: T
}

impl<S, T: RustEmbed + Clone> Layer<S> for Embed<T> {
    type Service = EmbedMiddleware<S, T>;

    fn layer(&self, inner: S) -> Self::Service {
        EmbedMiddleware { _assets: self.assets.clone(), inner }
    }
}

#[derive(Clone)]
pub struct EmbedMiddleware<S, T: RustEmbed + Clone> {
    _assets: T,
    inner: S,
}

impl<S, T> Service<Request<Body>> for EmbedMiddleware<S, T>
where
    S: Service<Request<Body>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
    T: RustEmbed + Clone,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static, Global>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        let path = request.uri().path().trim_start_matches('/');
        let Some(asset) = T::get(path) 
            else { return Box::pin(self.inner.call(request)) };
        Box::pin(async move {
            let response = Response::builder();
            let asset_etag = hex::encode(asset.metadata.sha256_hash());
            if let Some(request_etag) = request.headers().get(http::header::IF_NONE_MATCH) {
                if request_etag.as_bytes() == asset_etag.as_bytes() {
                    // respond with empty 304 to notify that file did not change
                    return Ok(response
                        .status(StatusCode::NOT_MODIFIED)
                        .body(Body::empty())
                        .unwrap());
                }
            }
            Ok(response
                .header(header::ETAG, asset_etag)
                .header(header::CONTENT_TYPE, asset.metadata.mimetype())
                .body(Body::from(asset.data))
                .unwrap())
        })
    }
}