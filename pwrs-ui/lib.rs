#![feature(allocator_api)]

use axum::{
    response::Response,
    body::{Body, HttpBody},
    http::Request,
};
use tower::{Service, Layer};
use std::{task::{Context, Poll}, pin::Pin, future::Future, alloc::Global};
use maud::{Markup, PreEscaped};

pub async fn add_html_content_type(
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

#[derive(Clone)]
pub struct Htmxify {
    pub wrapper: &'static fn(Markup) -> Markup
}

impl Htmxify {
    pub fn wrap(wrapper: &'static fn(Markup) -> Markup) -> Self {
        Self { wrapper }
    }
}

impl<S> Layer<S> for Htmxify {
    type Service = HtmxMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        HtmxMiddleware { wrapper: self.wrapper, inner }
    }
}

#[derive(Clone)]
pub struct HtmxMiddleware<S> {
    wrapper: &'static fn(Markup) -> Markup,
    inner: S,
}

impl<S> Service<Request<Body>> for HtmxMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static, Global>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        let is_htmx_request = request.headers().get("HX-Request").is_some();
        let future = self.inner.call(request);
        let wrapper = self.wrapper;
        Box::pin(async move {
            let response: Response = future.await?;
            let (mut parts, mut content) = response.into_parts();
            let mut buf = Vec::with_capacity(content.size_hint().lower() as usize);
            while let Some(chunk) = content.data().await {
                bytes::BufMut::put(&mut buf, chunk.unwrap());
            }
            let content = std::string::String::from_utf8(buf).unwrap();
            let content = if is_htmx_request {
                PreEscaped(content)
            } else {
                wrapper(PreEscaped(content))
            };
            let body = axum::body::Body::from(content.0);
            parts.headers.remove(http::header::CONTENT_LENGTH);
            let response = axum::response::Response::from_parts(parts, body);
            Ok(response)
        })
    }
}