mod analytics;
pub use analytics::*;

use crate::{host::LOG, *};

state!(DEPLOYED: bool = {
    env::var("DEPLOYED").map(|v| v == "true").unwrap_or(false)
});

pub async fn deploy() -> impl IntoResponse {
    if *DEPLOYED {
        return StatusCode::FORBIDDEN;
    }

    info!("Initiated deployment");

    let Ok(Ok(binary_path)) = tokio::task::spawn_blocking(move || build_linux_binary()).await
    else {
        return StatusCode::EXPECTATION_FAILED;
    };

    if let Err(e) = remote_update(&binary_path).await {
        error!("Failed to update the server: {e}");
        StatusCode::EXPECTATION_FAILED
    } else {
        StatusCode::OK
    }
}

pub async fn page() -> Markup {
    let tables = html! {
        @if DB_SCHEMA.tables().len() > 0 {
            h2{"DB explorer"}
        }
        @for table in DB_SCHEMA.tables() {
            h3 {(table.name())}
            div."loader" hx-get=(table.path()) hx-trigger="load" hx-swap="outerHTML" hx-target="this" {}
        }
    };
    let schedule_running_tasks = SCHEDULE
        .running_tasks
        .load(std::sync::atomic::Ordering::Relaxed);

    let schedule_msg = match schedule_running_tasks {
        0 => "".to_owned(),
        n => format!("schedule is running {n} tasks"),
    };

    html! {(DOCTYPE) (Head::with_title("Prest Admin"))
        body."max-w-screen-md lg:max-w-screen-lg container md:mx-auto" {
            nav."navbar bg-base-200 shadow-lg rounded-box my-4"{
                ."navbar-start md:gap-2" {
                    a."btn btn-ghost" href="/" {"BACK HOME"}
                    @if !*DEPLOYED {
                        button."btn btn-ghost" hx-get="/admin/deploy" hx-target="this" hx-swap="outerHTML" {"DEPLOY"}
                    }
                }
                ."navbar-center" {(schedule_msg)}
                ."navbar-end"{."dropdown dropdown-bottom dropdown-end"{
                    ."btn btn-ghost btn-circle" tabindex="0" role="button" {
                        svg."h-5 w-5" style="transform: scale(-1,1)" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                            path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h7" {}
                        }
                    }
                    ul."menu menu-md dropdown-content mt-3 z-10 p-2 bg-base-300 shadow-xl rounded-box w-52" tabindex="0" {
                        @if cfg!(feature = "auth") {
                            li { a href="/admin/shutdown" hx-boost="true" hx-target="body" hx-confirm="Are you sure you wish to completely shut down the app?" {"Shutdown"}}
                        }

                    }
                }}
            }
            main."view-transition mx-auto p-4 prose lg:prose-l lg:w-[1024px] lg:max-w-[1024px] [overflow-wrap:anywhere]" {
                span hx-get="/admin/analytics" hx-trigger="load delay:0.1s" hx-target="this" hx-swap="outerHTML" {}
                (tables)
                h2{"Latest logs"}
                div."w-full" hx-get="/admin/logs" hx-trigger="load delay:1s" hx-target="this" hx-swap="outerHTML" {}
            }
            ."menu menu-horizontal w-full items-center justify-center bg-base-200 rounded-box mb-4 mx-auto" {
                ."font-mono" {"powered by prest"}
                /*
                ."font-mono" {"v"(*PREST_VERSION)}
                ."ml-4 mr-2" {"made by Egor Dezhic"}
                a."btn btn-ghost btn-circle" href="https://twitter.com/eDezhic" target="_blank" {(PreEscaped(include_str!("../icons/twitter.svg")))}
                a."btn btn-ghost btn-circle" href="https://edezhic.medium.com" target="_blank" {(PreEscaped(include_str!("../icons/medium.svg")))}
                a."btn btn-ghost btn-circle" href="mailto:edezhic@gmail.com" target="_blank" {(PreEscaped(include_str!("../icons/email.svg")))}
                 */
            }
            (Scripts::default())
        }
    }
}

pub async fn logs() -> Markup {
    let logs = &LOG.read().unwrap();
    let latest_logs: Vec<PreEscaped<String>> = logs
        .lines()
        .rev()
        .take(100)
        .map(|log| PreEscaped(log.to_owned()))
        .collect();

    html! {
        div."w-full" hx-get="/admin/logs" hx-trigger="load delay:1s" hx-target="this" hx-swap="outerHTML"
            {@for log in latest_logs {p style="margin:0 !important"{(log)}}
        }
    }
}

pub fn db_routes() -> Router {
    let mut router = Router::new();
    for table in DB_SCHEMA.tables() {
        router = router.route(table.path(), get(|| async {
            let schema = table.schema();
            let values = table.get_all();
            let mut rows = vec![];
            for row_values in values {
                let mut cells = vec![];
                let key_selector = format!("a{}", row_values[0].clone());
                let inputs_classname = format!(".{key_selector}");

                for (schema, value) in std::iter::zip(schema, row_values) {
                    let input_type = match schema.glue_type {
                            "BOOLEAN" => "checkbox",
                            t if t.starts_with("UINT") || t.starts_with("INT") || t.starts_with("F") => "number",
                            "U64" | "U8" | "F64" => "number",
                            "TEXT" | _ => "text",
                    };

                    let cell_class = match schema.key {
                        true => "hidden",
                        false => "text-center",
                    };

                    let input_class = match input_type {
                        "text" | "number" => "input input-bordered w-full",
                        "checkbox" => "checkbox",
                        _ => "",
                    };

                    let checked = match value.as_str() {
                        "true" => true,
                        _ => false,
                    };

                    let onchange = match input_type {
                        "checkbox" => Some("this.value = this.checked ? 'true' : 'false'"),
                        _ => None
                    };

                    let cell = html! {
                        td.(cell_class) {input.(input_class).(key_selector) onchange=[(onchange)] type=(input_type) name=(schema.name) value=(value) checked[checked] {}}
                    };
                    cells.push(cell);
                }
                rows.push(html!(tr #(key_selector) ."relative" { 
                    @for cell in cells {(cell)}
                    td."flex justify-around items-center" {
                        button hx-post=(table.path()) hx-swap="none" hx-include=(inputs_classname) type="submit" {"Save"}
                        button hx-delete=(table.path()) hx-swap="outerHtml" hx-target=(format!("#{key_selector}")) hx-include=(inputs_classname) type="submit" {"Delete"}   
                    }
                }))
            }
            html!(
                table."w-full" {
                    @let headers = table.schema().iter().filter(|c| !c.key).map(|c| c.name);
                    @for header in headers {th {(header)}} th{"Actions"}
                    @for row in rows {(row)}
                }
            )
        })).route(table.path(), post(|req: Request| async {
            table.save(req).await
        })).route(table.path(), delete(|req: Request| async {
            table.remove(req).await
        }));
    }
    router
}
