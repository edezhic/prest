use crate::*;
use super::*;
use std::task::{Context, Poll};

pub type NonHtmxRequestWrapper = fn(Markup) -> Markup;

#[derive(Clone)]
pub struct Htmxify {
    pub wrapper: NonHtmxRequestWrapper,
}

impl Htmxify {
    pub fn wrap(wrapper: NonHtmxRequestWrapper) -> Self {
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
    wrapper: NonHtmxRequestWrapper,
    inner: S,
}

impl<S> Service<Request<Body>> for HtmxMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
  
    type Response = S::Response;
    type Error = S::Error;
    type Future = futures_util::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

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
            let content = PreEscaped(content);
            let content = if is_htmx_request {
                content
            } else {
                wrapper(content)
            };
            let body = Body::from(content.0);
            parts.headers.remove(header::CONTENT_LENGTH);
            if !parts.headers.contains_key(header::CONTENT_TYPE) {
                parts.headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html"));
            }
            let response = Response::from_parts(parts, body);
            Ok(response)
        })
    }
}
