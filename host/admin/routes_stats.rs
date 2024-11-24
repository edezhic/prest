use crate::{analytics::RouteStats, *};

pub(crate) async fn full() -> impl IntoResponse {
    let routes_stats = RouteStats::find_all();
    let mut total_path_hits = 0;

    let stats_view: Vec<(String, u64, String, Vec<(u16, u64)>, bool)> =  routes_stats.into_iter().map(|rs| {
        let statuses: Vec<(u16, u64)> = rs
            .statuses
            .into_iter()
            .filter(|(status, _)| *status != 200 && *status != 304)
            .collect();
        if !rs.is_asset {
            total_path_hits += rs.hits;
        }
        let avg_latency = format!("{:.3}ms", rs.avg_latency);
        (rs.path, rs.hits, avg_latency, statuses, rs.is_asset)
    }).collect();

    let (path_stats, asset_stats): (Vec<_>, Vec<_>) =
        stats_view.into_iter().partition(|r| { !r.4 });

    html! {
        $"font-bold text-lg" {"Routes stats (total hits: "(total_path_hits)"*)"}
        $"italic text-xs" {"*only counts requests to the server, static pages like blog's are served primarily by the Service Worker and aren't reflected here"}
        table $"w-full text-sm font-mono" {
            @for route in path_stats {
                tr {
                    td $"w-1/4"{(route.0)}
                    td $"w-1/4"{(route.1)}
                    td $"w-1/4"{(route.2)}
                    td $"w-1/4"{
                        @for (status, hits) in route.3 {
                            (status)"("(hits)")"
                        }
                    }
                }
            }
        }
        $"font-bold text-lg" {"Assets"}
        table $"w-full text-sm font-mono" {
            @for route in asset_stats {
                tr {
                    td $"w-1/4"{(route.0)}
                    td $"w-1/4"{(route.1)}
                    td $"w-1/4"{(route.2)}
                    td $"w-1/4"{
                        @for (status, hits) in route.3 {
                            (status)"("(hits)")"
                        }
                    }
                }
            }
        }
    }
}

