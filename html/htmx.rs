use futures::Future;

use crate::*;
use std::task::{Context, Poll};

/// Convenience trait to easily add [`HtmxLayer`] to the [`Router`] 
pub trait HtmxRouting<F> {
    fn wrap_non_htmx(self, wrapper: F) -> Self;
}
impl<F, MF> HtmxRouting<F> for Router 
where 
    F: Fn(Markup) -> MF + Clone + Send + 'static,
    MF: Future<Output = Markup> + Send
{
    fn wrap_non_htmx(self, wrapper: F) -> Self {
        self.route_layer(HtmxLayer::wrap(wrapper))
    }
}

/// Layer that modifies non-HTMX requests with the provided [`Fn`]
/// 
/// Function or closure must take a single [`Markup`] argument and return [`Markup`]
/// 
/// Can be used like this: `router.layer(HtmxLayer::wrap(|content| html!{body {(content)}}))`
/// 
/// It also sets a proper html content type header and disables caching for htmx responses
#[derive(Clone)]
pub struct HtmxLayer<F> {
    pub wrapper: F,
}

impl<F> HtmxLayer<F> {
    pub fn wrap(wrapper: F) -> Self {
        Self { wrapper }
    }
}

impl<S, F> Layer<S> for HtmxLayer<F>
where
    F: Clone,
{
    type Service = HtmxMiddleware<S, F>;

    fn layer(&self, inner: S) -> Self::Service {
        HtmxMiddleware {
            wrapper: self.wrapper.clone(),
            inner,
        }
    }
}

/// Underlying middleware that powers [`HtmxLayer`] layer
#[doc(hidden)]
#[derive(Clone)]
pub struct HtmxMiddleware<S, F> {
    wrapper: F,
    inner: S,
}

impl<S, F, MF> Service<Request<Body>> for HtmxMiddleware<S, F>
where
    S: Service<Request<Body>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
    F: Fn(Markup) -> MF + Send + Clone + 'static,
    MF: Future<Output = Markup> + Send
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        let not_htmx_request = request.headers().get("HX-Request").is_none();
        let future = self.inner.call(request);
        let wrapper = self.wrapper.clone();
        Box::pin(async move {
            let (mut parts, body) = future.await?.into_parts();
            parts
                .headers
                .insert(header::CONTENT_TYPE, HeaderValue::from_static("text/html"));
            
            if not_htmx_request {
                parts.headers.remove(header::CONTENT_LENGTH);
                let mut buf = Vec::with_capacity(body.size_hint().lower() as usize);
                let mut body_stream = body.into_data_stream();
                while let Some(Ok(chunk)) = body_stream.next().await {
                    bytes::BufMut::put(&mut buf, chunk);
                }
                let content = std::string::String::from_utf8(buf).unwrap();
                let content_future = wrapper(PreEscaped(content));
                let content = content_future.await;
                let body = Body::from(content.0);
                let response = Response::from_parts(parts, body);
                Ok(response)
            } else {
                parts
                    .headers
                    .insert(header::CACHE_CONTROL, HeaderValue::from_static("max-age=0, no-cache, must-revalidate, proxy-revalidate"));
                Ok(Response::from_parts(parts, body))
            }
        })
    }
}
