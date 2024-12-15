use crate::{host::internal_request, *};

use http::Method;
use pin_project_lite::pin_project;
use std::{collections::HashMap, task::ready, time::Instant};
use tracing::Span;

/// Describes collected stats for some path
#[derive(Debug, Table, Serialize, Deserialize)]
pub struct RouteStat {
    pub path: String,
    pub method_hits_and_latency: HashMap<String, (u64, f64)>,
    pub is_asset: bool,
}

impl RouteStat {
    pub fn record(req_method: Method, path: String, latency: f64) {
        let req_method = req_method.to_string();
        if let Some(mut stats) = RouteStat::find_by_path(&path) {
            let entry = stats.method_hits_and_latency.entry(req_method).or_default();

            let updated_hits = entry.0 + 1;

            let updated_avg_latency = (entry.0 as f64 * entry.1 + latency) / (updated_hits as f64);

            *entry = (updated_hits, updated_avg_latency);

            if let Err(e) = stats.save() {
                warn!(target:"analytics", "Failed to update stats: {e}");
            }
        } else {
            let is_asset =
                mime_guess::from_path(&path).first().is_some() || path.ends_with(".webmanifest");

            let mut mhal = HashMap::new();
            mhal.insert(req_method, (1, latency));

            let stats = RouteStat {
                path,
                method_hits_and_latency: mhal,
                is_asset,
            };

            if let Err(e) = stats.save() {
                warn!(target:"analytics", "Failed to save new stats: {e}");
            }
        }
    }
}

fn record_response_metrics(
    resp: &Response,
    latency: std::time::Duration,
    _span: &Span,
    req_method: Method,
    req_path: String,
) {
    let latency = latency.as_secs_f64() * 1000.0; // into millis
    let short_latency = format!("{latency:.3}");

    let status = resp.status();

    let boring_resp = matches!(
        status,
        StatusCode::NOT_MODIFIED | StatusCode::METHOD_NOT_ALLOWED | StatusCode::NOT_FOUND
    );

    match boring_resp {
        true => trace!(target: "response", latency = %short_latency, code = %status.as_u16()),
        false => {
            debug!(target: "response", latency = %short_latency, code = %status.as_u16());
            RT.spawn_blocking(move || {
                RouteStat::record(req_method, req_path, latency);
            });
        }
    }
}

/// Layer that collects analytics
#[derive(Clone)]
pub struct AnalyticsLayer;

impl AnalyticsLayer {
    pub fn init() -> Self {
        RouteStat::migrate();
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
        let span = make_span(&req);
        let req_method = req.method().clone();
        let req_path = req.uri().path().to_owned();

        let future = {
            let _guard = span.enter();
            self.inner.call(req)
        };

        ResponseFuture {
            inner: future,
            span,
            start,
            req_method: Some(req_method),
            req_path: Some(req_path),
            internal_req,
        }
    }
}

fn make_span(request: &Request) -> Span {
    let method = request.method().as_str();
    let uri = request.uri();
    let path = uri.path();
    let uri = if path.starts_with("/auth/") && uri.query().is_some() {
        path.to_owned() + "?[redacted]"
    } else {
        uri.to_string()
    };

    tracing::debug_span!("http", method, uri)
}

pin_project! {
    pub struct ResponseFuture<F> {
        #[pin]
        pub(crate) inner: F,
        pub(crate) span: Span,
        pub(crate) start: Instant,
        pub(crate) req_method: Option<Method>,
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

        match result {
            Ok(res) => {
                if !*this.internal_req {
                    record_response_metrics(
                        &res,
                        this.start.elapsed(),
                        this.span,
                        this.req_method.take().unwrap(),
                        this.req_path.take().unwrap(),
                    );
                }

                Poll::Ready(Ok(res))
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}
