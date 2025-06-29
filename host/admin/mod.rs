use crate::*;

mod analytics;
mod db;
mod logs;
mod monitoring;
mod remote;
mod schedule;

const ADMIN_SVG: PreEscaped<&str> = PreEscaped(include_str!("assets/admin.svg"));
const DB_SVG: PreEscaped<&str> = PreEscaped(include_str!("assets/db.svg"));
const LOGS_SVG: PreEscaped<&str> = PreEscaped(include_str!("assets/logs.svg"));
const ANALYTICS_SVG: PreEscaped<&str> = PreEscaped(include_str!("assets/analytics.svg"));
const LOADER_SVG: PreEscaped<&str> = PreEscaped(include_str!("assets/loader.svg"));

pub(crate) async fn routes() -> Router {
    route(
        "/",
        get(|| async {
            ok(html!(
                (monitoring::container().await?)
                a get="/admin/remote/state" trigger="load" swap-this {}
                (logs::info_explorer().await)
            ))
        }),
    )
    .route("/monitoring", get(monitoring::container))
    .route("/latest_info", get(logs::info_explorer))
    .route("/latest_info/:offset", get(logs::info))
    .route("/traces", get(logs::traces_explorer))
    .route("/schedule", get(schedule::full))
    .route("/analytics", get(analytics::full))
    .route("/db", get(db::db_page))
    .nest("/remote", remote::routes())
    .wrap_non_htmx(into_page)
    .nest("/db", db::table_routes())
    .route("/db/schema", get(db::schema))
    .route("/traces/:period", get(logs::traces))
    .route("/monitoring/data", get(monitoring::data))
}

async fn into_page(content: Markup) -> impl IntoResponse {
    html! {(DOCTYPE) html $"bg-stone-800 font-sans text-gray-300" {
        (Head::with_title("Prest Admin"))
        body $"max-w-screen-md lg:max-w-screen-lg md:mx-auto" {
            nav replace-url into="main" $"bg-stone-900 my-4 p-5 shadow-lg rounded-full items-center flex gap-6 w-min mx-auto" {
                a href="/" boost="false" {(home_svg())}
                button get="/admin" {$"w-6" {(ADMIN_SVG)}}
                button get="/admin/analytics" {$"w-6" {(ANALYTICS_SVG)}}
                button get="/admin/traces" {$"w-6" {(LOGS_SVG)}}
                @if DB.custom_schemas().len() > 0 {
                    button get="/admin/db" {$"w-6" {(DB_SVG)}}
                }
            }
            main $"opacity-80 mx-auto p-4 gap-4 flex flex-col text-sm lg:text-base leading-loose" {
                (content)
            }
            (Scripts::default().include("/traces.js").include("/db.js").module("/stats.js").css("/admin.css"))
        }
    }}
}

fn home_svg() -> Markup {
    html!(
        svg $"w-6" viewBox="0 0 16 16" fill="none" {
            path d="M1 6V15H6V11C6 9.89543 6.89543 9 8 9C9.10457 9 10 9.89543 10 11V15H15V6L8 0L1 6Z" fill="currentColor" {}
        }
    )
}
