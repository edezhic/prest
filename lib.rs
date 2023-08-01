#![feature(allocator_api, type_alias_impl_trait)]

pub use axum::{
    self,
    body::{Body, HttpBody},
    extract::Request,
    response::Response,
    Router,
};
pub use bytes;
pub use http::{self, header, StatusCode};
pub use maud::{Markup, PreEscaped};
pub use tower::{Layer, Service};

#[macro_export]
macro_rules! render {
    ($template: ident) => {
        axum::routing::get(|| async {
            (
                [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
                $template::render().0,
            )
        })
    };
}

use std::{
    alloc::Global,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

type Wrapper = fn(Markup) -> Markup;

#[derive(Clone)]
pub struct Htmxify {
    pub wrapper: Wrapper,
}

impl Htmxify {
    pub fn wrap(wrapper: Wrapper) -> Self {
        Self { wrapper }
    }
}

impl<S> Layer<S> for Htmxify {
    type Service = HtmxMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        HtmxMiddleware {
            wrapper: self.wrapper,
            inner,
        }
    }
}

#[derive(Clone)]
pub struct HtmxMiddleware<S> {
    wrapper: Wrapper,
    inner: S,
}

impl<S> Service<Request<Body>> for HtmxMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static, Global>>;

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
