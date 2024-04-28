use crate::{host::LOG, *};

pub async fn page() -> Markup {
    let routes_stats = RouteStats::find_all();
    let routes_info = html! {
        h2{"Routes stats"}
        table."w-full" {
            @for route in routes_stats {
                tr {
                    td{(route.path)}
                    td{(route.hits)}
                    td{
                        @for (status, hits) in route.statuses {
                            (status)"("(hits)")"
                        }
                    }
                }
            }
        }
    };
    let tables = html! {
        h2{"DB explorer"}
        @for table in DB_SCHEMA.tables() {
            h3 {(table.name())}
            div."loader" hx-get=(table.path()) hx-trigger="load" hx-swap="outerHTML" hx-target="this" {}
        }
    };
    let schedule_running_tasks = SCHEDULE
        .running_tasks
        .load(std::sync::atomic::Ordering::Relaxed);

    let logs = &LOG.read().unwrap();
    let latest_logs: Vec<PreEscaped<String>> = logs
        .lines()
        .rev()
        .take(100)
        .map(|log| PreEscaped(log.to_owned()))
        .collect();

    html! {(DOCTYPE) (Head::with_title("Prest Admin"))
        body."max-w-screen-sm mx-auto mt-12 flex flex-col items-center gap-y-8" {
            p{"Schedule is running "(schedule_running_tasks)" tasks right now"}
            (routes_info)
            (tables)
            h2{"Latest logs"}
            div."w-full" {@for log in latest_logs {p{(log)}}}
            (Scripts::default())
        }
    }
}
