use crate::*;
use super::internal_req;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Table, Serialize, Deserialize)]
pub struct RouteStats {
    pub path: String,
    pub hits: i64,
    pub statuses: HashMap<u16, u64>,
}

/// Layer that modifies non-HTMX requests with the provided [`Fn`]
///
/// Function or closure must take a single [`Markup`] argument and return [`Markup`]
///
/// Can be used like this: `router.layer(HtmxLayer::wrap(|content| html!{body {(content)}}))`
///
/// It also sets a proper html content type header and disables caching for htmx responses
#[derive(Clone)]
pub struct AnalyticsLayer {
    //pub wrapper: OptionF,
}

impl AnalyticsLayer {
    pub fn init() -> Self {
        RouteStats::migrate();
        Self { }
    }
}

impl<S> Layer<S> for AnalyticsLayer
{
    type Service = AnalyticsMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AnalyticsMiddleware {
            inner,
        }
    }
}

/// Underlying middleware that powers [`HtmxLayer`] layer
#[doc(hidden)]
#[derive(Clone)]
pub struct AnalyticsMiddleware<S> {
    inner: S,
}

use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use std::boxed::Box;

impl<S> Service<Request<Body>> for AnalyticsMiddleware<S>
where
    S: Service<Request<Body>, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<
        Box<dyn Future<Output = std::result::Result<Self::Response, Self::Error>> + Send + 'static>,
    >;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<Body>) -> Self::Future {
        let path = match internal_req(&request) {
            true => None,
            false => Some(request.uri().path().to_owned()),
        };
        let future = self.inner.call(request);
        Box::pin(async move {
            let response = future.await?;

            if let Some(path) = path {
                let status = response.status().as_u16();
                let mut stats = RouteStats::find_by_path(&path).unwrap_or(RouteStats {
                    path,
                    hits: 0,
                    statuses: HashMap::new(),
                });
                stats.hits += 1;
                stats
                    .statuses
                    .entry(status)
                    .and_modify(|v| *v += 1)
                    .or_insert(1);
                if let Err(e) = stats.save() {
                    warn!("Failed to update stats: {e}");
                }
            }

            Ok(response)
        })
    }
}
