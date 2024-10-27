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
            $"font-bold text-lg" {"DB explorer"}
        } @else {
            $"font-bold text-lg" {"No initialized tables"}
        }
        @for table in DB_SCHEMA.tables() {
            $"font-bold text-lg" {(table.name())}
            div ."loader" hx-get=(table.path()) hx-trigger="load" hx-swap="outerHTML" hx-target="this" {}
        }
    };
    let running_scheduled_tasks = RT
        .running_scheduled_tasks
        .load(std::sync::atomic::Ordering::Relaxed);

    let schedule_msg = match running_scheduled_tasks {
        0 => "".to_owned(),
        n => format!("schedule is running {n} tasks"),
    };

    let content = html! {(DOCTYPE) html $"bg-stone-800 font-sans text-[#bbc4d4]" _="on click remove .open from #menu" {
        (Head::with_title("Prest Admin"))
        body $"max-w-screen-md lg:max-w-screen-lg md:mx-auto" {
            nav $"bg-stone-900 my-4 p-5 shadow-lg rounded-full grid grid-cols-3 items-center" {
                $"flex gap-6" {
                    a."btn btn-ghost" href="/" hx-boost="false" {"Back home"}
                    (DEPLOY.button())
                }

                $"text-center" {(schedule_msg)}

                $"flex justify-end" {
                    $"hover:text-white" _="on click add .open to #menu halt" {
                        svg $"h-5 w-5 scale-[-1,1]" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                            path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h7" {}
                        }
                    }

                    div id="menu" $"absolute bg-stone-950 z-10 top-8 px-4 truncate shadow-xl rounded-xl w-52" {
                        style {"
                            #menu { max-height: 0px } #menu.open { max-height: 1000px } 
                            #menu a { display: flex; align-items: center; padding: 0.25rem 0 0.25rem 0.5rem; border-radius: 1rem; }
                            #menu a:hover { background-color: #292524 }
                        "}
                        $"py-4 flex flex-col gap-2 text-xs" {
                            @if cfg!(feature = "auth") {
                                a href="/admin/shutdown" hx-boost="true" hx-target="body"
                                    hx-confirm="Are you sure you wish to completely shut down the app?" {"Shutdown"}
                            }
                        }
                    }
                }
            }
            main $"opacity-80 mx-auto p-4 gap-3 flex flex-col text-sm lg:text-base leading-loose" {
                span hx-get="/admin/analytics" hx-trigger="load delay:0.1s" hx-target="this" hx-swap="outerHTML" {}
                (tables)
                $"font-bold text-lg" {"Latest logs"}
                div $"w-full" hx-get="/admin/logs" hx-trigger="load delay:0.1s" hx-target="this" hx-swap="outerHTML" {}
                button $"font-bold text-lg" hx-get="/admin/debug_logs" hx-target="this" hx-swap="outerHTML" {"See debug logs"}
            }
            $"flex items-center justify-evenly p-4 w-full bg-stone-900 rounded-full mb-4 mx-auto text-xs lg:text-base" {
                $"text-sm" {"powered by prest"}
            }
            (Scripts::default())
        }
    }};

    (
        (
            HxRetarget::from("body"),
            HxReswap::from(SwapOption::OuterHtml),
        ),
        content,
    )
}

pub(crate) async fn logs() -> Markup {
    let logs: Vec<_> = LOG
        .read_last_lines(30)
        .into_iter()
        .map(|log| PreEscaped(log))
        .collect();

    html! {
        div $"w-full" hx-get="/admin/logs" hx-trigger="load delay:1s" hx-target="this" hx-swap="outerHTML"
            {@for log in logs {p style="margin:0 !important"{(log)}}
        }
    }
}

pub(crate) async fn debug_logs() -> Markup {
    let logs: Vec<_> = DEBUG_LOG
        .read_last_lines(300)
        .into_iter()
        .map(|log| PreEscaped(log))
        .collect();

    html! {
        div $"w-full" hx-get="/admin/debug_logs" hx-trigger="load delay:1s" hx-target="this" hx-swap="outerHTML"
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
