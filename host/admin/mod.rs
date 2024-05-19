mod analytics;
pub use analytics::*;

mod db_editor;
pub use db_editor::*;

mod deploy;
pub use deploy::*;

use crate::{host::LOG, *};

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
                    a."btn btn-ghost" href="/" {"Back home"}
                    (DEPLOY.button())
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

#[allow(dead_code)]
pub async fn shutdown() -> impl IntoResponse {
    SHUTDOWN.initiate();
    html! {
        h1 {"Shutdown has been initiated"}
    }
}
