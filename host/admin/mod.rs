use crate::{host::LOGS, *};

mod routes_stats;

mod db_editor;
pub(crate) use db_editor::*;

mod remote;

pub(crate) fn routes() -> Router {
    route("/", get(page))
        .route("/latest_info", get(latest_info))
        .route("/traces", get(traces))
        .route("/system_stats", get(system_stats))
        .route("/schedule_stats", get(schedule_stats))
        .route("/routes_stats", get(routes_stats::full))
        .nest("/remote", remote::routes())
        .nest("/db", db_routes())
}

async fn page() -> impl IntoResponse {
    let tables = html! {
        @if DB_SCHEMA.tables().len() > 0 {
            $"font-bold text-lg" {"DB explorer"}
        }
        @for table in DB_SCHEMA.tables() {
            $"font-bold text-lg" {(table.name())}
            div ."loader" get=(table.full_path()) trigger="load" swap-full into="this" {}
        }
    };

    let content = html! {(DOCTYPE) html $"bg-stone-800 font-sans text-[#bbc4d4]" {
        (Head::with_title("Prest Admin"))
        body $"max-w-screen-md lg:max-w-screen-lg md:mx-auto" {
            nav $"bg-stone-900 my-4 p-5 shadow-lg rounded-full grid grid-cols-3 items-center" {
                $"flex gap-6" {
                    a."btn btn-ghost" href="/" boost="false" {"Back home"}
                }
                span get="/admin/schedule_stats" trigger="load delay:1s" into="this" swap-full {}
            }
            main $"opacity-80 mx-auto p-4 gap-3 flex flex-col text-sm lg:text-base leading-loose" {
                span get="/admin/system_stats" trigger="load" into="this" swap-full {}
                span get="/admin/remote" trigger="load" into="this" swap-full {}
                span get="/admin/routes_stats" trigger="load" into="this" swap-full {}
                (tables)
                $"font-bold text-lg" {"Latest info"}
                $"w-full" get="/admin/latest_info" trigger="load" into="this" swap-full {}
                button get="/admin/traces" into="this" swap-full {"Load full traces"}
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

async fn latest_info() -> Markup {
    let logs: Vec<_> = LOGS
        .latest_info(30)
        .into_iter()
        .map(|log| PreEscaped(log))
        .collect();

    html! {
        div $"w-full font-mono text-sm leading-snug" get="/admin/latest_info" trigger="load delay:1s" into="this" swap-full
            {@for log in logs {p style="margin:0 !important"{(log)}}
        }
    }
}

async fn traces() -> Markup {
    html! {
        code $"w-full font-mono text-xs leading-snug" get="/admin/traces" trigger="load delay:3s" into="this" swap-full
            {(PreEscaped(LOGS.traces()))}
    }
}

async fn system_stats() -> Markup {
    let sys = SYSTEM_INFO.system.read().await;
    let Some(current) = sys.process(SYSTEM_INFO.app_pid) else {
        return html!();
    };

    let total_ram = format!(
        "used / total memory: {} / {} MBs",
        sys.used_memory().div_ceil(1_000_000),
        sys.total_memory().div_ceil(1_000_000)
    );
    
    let memory = current.memory().div_ceil(1_000_000); // into MBs

    let cpu_usage = current.cpu_usage() / SYSTEM_INFO.cores as f32;

    let all_cpu_usage = sys.global_cpu_usage();

    let current_cpu = format!("app / total CPU: {cpu_usage:.2} / {all_cpu_usage:.2}");
    let current_ram = format!("app memory: {memory} MBs");

    let current_disk_written = current.disk_usage().total_written_bytes.div_ceil(1_000_000);

    let current_disk = format!("written on disk: {current_disk_written} MBs");

    html!(
        $"w-full" get="/admin/system_stats" trigger="load delay:1s" into="this" swap-full {
            $"font-bold text-lg" {"System stats"}
            p{(current_cpu)}
            p{(current_ram)}
            p{(total_ram)}
            p{(current_disk)}
        }
    )
}

async fn schedule_stats() -> Markup {
    let running_scheduled_tasks = RT
        .running_scheduled_tasks
        .load(std::sync::atomic::Ordering::Relaxed);

    let schedule_msg = match running_scheduled_tasks {
        0 => "no scheduled tasks running atm".to_owned(),
        n => format!("{n} scheduled tasks running"),
    };

    html!(
        $"text-center" get="/admin/schedule_stats" trigger="load delay:1s" into="this" swap-full {(schedule_msg)}
    )
}
