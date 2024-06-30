mod analytics;
pub use analytics::*;

mod db_editor;
pub(crate) use db_editor::*;

mod deploy;
pub(crate) use deploy::*;
use host::DEBUG_LOG;

use crate::{host::LOG, *};

pub(crate) async fn page() -> impl IntoResponse {
    let tables = html! {
        @if DB_SCHEMA.tables().len() > 0 {
            h2{"DB explorer"}
        } @else {
            h3{"No initialized tables"}
        }
        @for table in DB_SCHEMA.tables() {
            h3 {(table.name())}
            div."loader" hx-get=(table.path()) hx-trigger="load" hx-swap="outerHTML" hx-target="this" {}
        }
    };
    let running_scheduled_tasks = RT
        .running_scheduled_tasks
        .load(std::sync::atomic::Ordering::Relaxed);

    let schedule_msg = match running_scheduled_tasks {
        0 => "".to_owned(),
        n => format!("schedule is running {n} tasks"),
    };

    let content = html! {(DOCTYPE) (Head::with_title("Prest Admin"))
        body."max-w-screen-md lg:max-w-screen-lg container md:mx-auto" {
            nav."navbar bg-base-200 shadow-lg rounded-box my-4"{
                ."navbar-start md:gap-2" {
                    a."btn btn-ghost" href="/" hx-boost="false" {"Back home"}
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
                div."w-full" hx-get="/admin/logs" hx-trigger="load delay:0.1s" hx-target="this" hx-swap="outerHTML" {}
                h2{"Debug logs"}
                div."w-full" hx-get="/admin/debug_logs" hx-trigger="load delay:1s" hx-target="this" hx-swap="outerHTML" {}
            }
            ."menu menu-horizontal w-full items-center justify-center bg-base-200 rounded-box mb-4 mx-auto" {
                ."font-mono" {"powered by prest"}
            }
            (Scripts::default())
        }
    };

    (
        (
            HxRetarget::from("body"),
            HxReswap::from(SwapOption::OuterHtml),
        ),
        content,
    )
}

pub(crate) async fn logs() -> Markup {
    let logs: Vec<_> = LOG.read_last_lines(100).into_iter()
        .map(|log| PreEscaped(log))
        .collect();

    html! {
        div."w-full" hx-get="/admin/logs" hx-trigger="load delay:1s" hx-target="this" hx-swap="outerHTML"
            {@for log in logs {p style="margin:0 !important"{(log)}}
        }
    }
}

pub(crate) async fn debug_logs() -> Markup {
    let logs: Vec<_> = DEBUG_LOG.read_last_lines(1000).into_iter()
        .map(|log| PreEscaped(log))
        .collect();

    html! {
        div."w-full" hx-get="/admin/debug_logs" hx-trigger="load delay:1s" hx-target="this" hx-swap="outerHTML"
            {@for log in logs {p style="margin:0 !important"{(log)}}
        }
    }
}

#[allow(dead_code)]
pub(crate) async fn shutdown() -> impl IntoResponse {
    SHUTDOWN.initiate();
    html! {
        h1 {"Shutdown has been initiated"}
    }
}
