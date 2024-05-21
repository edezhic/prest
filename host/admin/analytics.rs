use crate::host::{filter_request, filter_response};
use crate::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub(crate) async fn analytics() -> impl IntoResponse {
    let routes_stats = RouteStats::find_all();
    let (path_stats, asset_stats): (Vec<RouteStats>, Vec<RouteStats>) =
        routes_stats.into_iter().partition(|r| {
            mime_guess::from_path(&r.path).is_empty() && !r.path.ends_with(".webmanifest")
        });

    let mut total_path_hits = 0;
    let path_stats: Vec<_> = path_stats
        .into_iter()
        .map(|rs| {
            let statuses: Vec<(u16, u64)> = rs
                .statuses
                .into_iter()
                .filter(|(status, _)| *status != 200)
                .collect();
            total_path_hits += rs.hits;
            (rs.path, rs.hits, statuses)
        })
        .collect();

    let asset_stats: Vec<_> = asset_stats
        .into_iter()
        .map(|rs| {
            let statuses: Vec<(u16, u64)> = rs
                .statuses
                .into_iter()
                .filter(|(status, _)| *status != 200 && *status != 304)
                .collect();
            (rs.path, rs.hits, statuses)
        })
        .collect();

    html! {
        h2{"Routes stats"}
        p{b{"Total path hits: "(total_path_hits)}}
        table."w-full" {
            @for route in path_stats {
                tr {
                    td."w-1/2"{(route.0)}
                    td."w-1/4"{(route.1)}
                    td."w-1/4"{
                        @for (status, hits) in route.2 {
                            (status)"("(hits)")"
                        }
                    }
                }
            }
        }
        h3{"Assets"}
        table."w-full" {
            @for route in asset_stats {
                tr {
                    td."w-1/2"{(route.0)}
                    td."w-1/4"{(route.1)}
                    td."w-1/4"{
                        @for (status, hits) in route.2 {
                            (status)"("(hits)")"
                        }
                    }
                }
            }
        }
    }
}

/// Describes collected stats for some path
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
        Self {}
    }
}

impl<S> Layer<S> for AnalyticsLayer {
    type Service = AnalyticsMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AnalyticsMiddleware { inner }
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
        let path = match filter_request(&request) {
            true => None,
            false => Some(request.uri().path().to_owned()),
        };
        let future = self.inner.call(request);
        Box::pin(async move {
            let response = future.await?;
            let status = response.status().as_u16();

            if let Some(path) = path {
                let filtered = filter_response(&response);
                tokio::task::spawn_blocking(move || record_response(path, status, filtered));
            }

            Ok(response)
        })
    }
}

fn record_response(path: String, status: u16, filtered: bool) {
    if let Some(mut stats) = RouteStats::find_by_path(&path) {
        stats.hits += 1;
        stats
            .statuses
            .entry(status)
            .and_modify(|v| *v += 1)
            .or_insert(1);
        if let Err(e) = stats.save() {
            warn!("Failed to update stats: {e}");
        }
    } else {
        if !filtered {
            let mut stats = RouteStats {
                path,
                hits: 0,
                statuses: HashMap::new(),
            };
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
    }
}
