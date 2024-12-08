use crate::*;

mod db_editor;
mod logs;
mod remote;
mod routes_stats;
mod schedule_stats;
mod system_stats;

const ADMIN_SVG: PreEscaped<&str> = PreEscaped(include_str!("assets/admin.svg"));
const DB_SVG: PreEscaped<&str> = PreEscaped(include_str!("assets/db.svg"));
const LOGS_SVG: PreEscaped<&str> = PreEscaped(include_str!("assets/logs.svg"));
const ANALYTICS_SVG: PreEscaped<&str> = PreEscaped(include_str!("assets/analytics.svg"));
const LOADER_SVG: PreEscaped<&str> = PreEscaped(include_str!("assets/loader.svg"));
const EDIT_SVG: PreEscaped<&str> = PreEscaped(include_str!("assets/edit.svg"));
const DONE_SVG: PreEscaped<&str> = PreEscaped(include_str!("assets/done.svg"));
const DELETE_SVG: PreEscaped<&str> = PreEscaped(include_str!("assets/delete.svg"));

pub(crate) fn routes() -> Router {
    route(
        "/",
        get(|| async {
            ok(html!(
                a get="/admin/remote/state" trigger="load" swap-this {}
                (system_stats::full().await?)
                (logs::info_explorer().await)
            ))
        }),
    )
    .route("/system_stats", get(system_stats::full))
    .route("/latest_info", get(logs::info_explorer))
    .route("/latest_info/:offset", get(logs::info))
    .route("/traces", get(logs::traces_explorer))
    .route("/schedule_stats", get(schedule_stats::full))
    .route("/analytics", get(routes_stats::full))
    .nest("/remote", remote::routes())
    .nest("/db", db_editor::db_routes())
    .wrap_non_htmx(into_page)
    .route("/traces/:period", get(logs::traces))
}

async fn into_page(content: Markup) -> impl IntoResponse {
    let page = html! {(DOCTYPE) html $"bg-stone-800 font-sans text-gray-300" {
        (Head::with_title("Prest Admin"))
        body $"max-w-screen-md lg:max-w-screen-lg md:mx-auto" {
            nav replace-url swap="innerHTML" $"bg-stone-900 my-4 p-5 shadow-lg rounded-full items-center flex gap-6 w-min mx-auto" {
                a href="/" boost="false" {(home_svg())}
                button get="/admin" into="main" {div $"w-6" {(ADMIN_SVG)}}
                button get="/admin/analytics" into="main" {div $"w-6" {(ANALYTICS_SVG)}}
                button get="/admin/traces" into="main" {div $"w-6" {(LOGS_SVG)}}
                @if DB_SCHEMA.tables().len() > 0 {
                    button get="/admin/db" into="main" {div $"w-6" {(DB_SVG)}}
                }
            }
            main $"opacity-80 mx-auto p-4 gap-8 flex flex-col text-sm lg:text-base leading-loose" {
                (content)
            }
            (Scripts::default().include("/admin.js").css("/admin.css"))
        }
    }};

    (
        (
            HxRetarget::from("body"),
            HxReswap::from(SwapOption::OuterHtml),
        ),
        page,
    )
}

fn home_svg() -> Markup {
    html!(
        svg $"w-6" viewBox="0 0 16 16" fill="none" {
            path d="M1 6V15H6V11C6 9.89543 6.89543 9 8 9C9.10457 9 10 9.89543 10 11V15H15V6L8 0L1 6Z" fill="currentColor" {}
        }
    )
}
