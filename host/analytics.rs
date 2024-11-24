use crate::{host::internal_request, *};

use pin_project_lite::pin_project;
use std::{collections::HashMap, task::ready, time::Instant};
use tracing::Span;

/// Describes collected stats for some path
#[derive(Debug, Table, Serialize, Deserialize)]
pub struct RouteStats {
    pub path: String,
    pub hits: u64,
    pub statuses: HashMap<u16, u64>,
    pub avg_latency: f64,
    pub is_asset: bool,
}

fn record_response_metrics(
    resp: &Response,
    latency: std::time::Duration,
    _span: &Span,
    req_path: String,
    internal_req: bool,
) {
    let latency = latency.as_secs_f64() * 1000.0; // into millis
    let short_latency = format!("{latency:.3}");

    let status = resp.status();
    let boring_resp =
        status == StatusCode::NOT_MODIFIED || status == StatusCode::METHOD_NOT_ALLOWED;

    if boring_resp {
        trace!(kind = "response", latency_ms = %short_latency, code = %status);
    } else {
        debug!(kind = "response", latency_ms = %short_latency, code = %status);
    }

    if internal_req || status == StatusCode::NOT_FOUND {
        return;
    }

    RT.spawn_blocking(move || {
        let status = status.as_u16();
        if let Some(mut stats) = RouteStats::find_by_path(&req_path) {
            let updated_hits = stats.hits + 1;

            let updated_avg_latency =
                (stats.hits as f64 * stats.avg_latency + latency) / (updated_hits as f64);

            stats.hits = updated_hits;
            stats.avg_latency = updated_avg_latency;

            stats
                .statuses
                .entry(status)
                .and_modify(|v| *v += 1)
                .or_insert(1);

            if let Err(e) = stats.save() {
                warn!("Failed to update stats: {e}");
            }
        } else {
            let is_asset = mime_guess::from_path(&req_path).first().is_some()
                || req_path.ends_with(".webmanifest");

            let mut stats = RouteStats {
                path: req_path,
                hits: 1,
                statuses: HashMap::new(),
                avg_latency: latency,
                is_asset,
            };

            stats.statuses.insert(status, 1);

            if let Err(e) = stats.save() {
                warn!("Failed to save new stats: {e}");
            }
        }
    });
}

/// Layer that collects analytics
#[derive(Clone)]
pub struct AnalyticsLayer;

impl AnalyticsLayer {
    pub fn init() -> Self {
        RouteStats::migrate();
        Self
    }
}

impl<S> Layer<S> for AnalyticsLayer {
    type Service = AnalyticsMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AnalyticsMiddleware { inner }
    }
}

/// Underlying middleware that powers [`AnalyticsLayer`]
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
    S::Error: std::fmt::Display + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<std::result::Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let start = Instant::now();
        let internal_req = internal_request(&req);
        let span = make_span(&req, internal_req);
        let req_path = req.uri().path().to_owned();

        let future = {
            let _guard = span.enter();
            self.inner.call(req)
        };

        ResponseFuture {
            inner: future,
            span,
            start,
            req_path: Some(req_path),
            internal_req,
        }
    }
}

fn make_span(request: &Request, internal_req: bool) -> Span {
    let method = request.method().as_str();
    let uri = request.uri();
    let path = uri.path();
    let uri = if path.starts_with("/auth/") && uri.query().is_some() {
        path.to_owned() + "?[redacted]"
    } else {
        uri.to_string()
    };

    if internal_req {
        tracing::trace_span!("request", method, uri)
    } else {
        tracing::debug_span!("request", method, uri)
    }
}

pin_project! {
    pub struct ResponseFuture<F> {
        #[pin]
        pub(crate) inner: F,
        pub(crate) span: Span,
        pub(crate) start: Instant,
        pub(crate) req_path: Option<String>,
        pub(crate) internal_req: bool,
    }
}

impl<Fut, E> Future for ResponseFuture<Fut>
where
    Fut: Future<Output = Result<Response, E>>,
    E: std::fmt::Display + 'static,
{
    type Output = Result<Response, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _guard = this.span.enter();
        let result = ready!(this.inner.poll(cx));
        let latency = this.start.elapsed();

        match result {
            Ok(res) => {
                record_response_metrics(
                    &res,
                    latency,
                    this.span,
                    this.req_path.take().unwrap(),
                    *this.internal_req,
                );
                Poll::Ready(Ok(res))
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}
