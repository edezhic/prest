use crate::{analytics::RouteStat, *};

pub(crate) async fn full() -> Result<Markup> {
    let routes_stats = RouteStat::find_all()?;
    let mut total_path_hits = 0;

    type Stats = Vec<(Markup, Markup, u64, Markup)>;

    let mut path_stats: Stats = vec![];
    let mut asset_stats: Stats = vec![];

    for route in routes_stats {
        for (method, (hits, latency)) in route.method_hits_and_latency {
            let method = PreEscaped(method);
            let path = PreEscaped(route.path.clone());
            let latency = PreEscaped(format!("{:.3}ms", latency));

            let view = (method, path, hits, latency);

            if route.is_asset {
                asset_stats.push(view);
            } else {
                path_stats.push(view);
                total_path_hits += hits;
            }
        }
    }

    Ok(html! {
        a get="/admin/schedule_stats" trigger="load" swap-this {}
        $"font-bold text-lg" {"Routes stats (total hits: "(total_path_hits)"*)"}
        $"hidden md:block italic text-xs" {"*only counts requests to the server, static pages like blog's are served primarily by the Service Worker and aren't reflected here"}
        table $"w-full text-xs md:text-sm font-mono" {
            @for route in path_stats {
                tr {
                    td $"w-[17%]"{(route.0)}
                    td $"w-[53%]"{(route.1)}
                    td $"w-[10%]"{(route.2)}
                    td $"w-[20%]"{(route.3)}
                }
            }
        }
        $"font-bold text-lg" {"Assets"}
        table $"w-full text-xs md:text-sm font-mono" {
            @for route in asset_stats {
                tr {
                    td $"w-[17%]"{(route.0)}
                    td $"w-[53%]"{(route.1)}
                    td $"w-[10%]"{(route.2)}
                    td $"w-[20%]"{(route.3)}
                }
            }
        }
    })
}
