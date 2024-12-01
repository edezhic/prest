use crate::{host::LOGS, *};

pub(crate) async fn latest_info() -> Markup {
    let logs: Vec<_> = LOGS
        .latest_info(30)
        .into_iter()
        .map(|log| PreEscaped(log))
        .collect();

    html! {
        div $"w-full font-mono text-xs md:text-sm leading-snug" get="/admin/latest_info" trigger="load delay:1s" into="this" swap-full
            {@for log in logs {p style="margin:0 !important"{(log)}}
        }
        button get="/admin/traces" into="this" swap-full {"Load full traces"}
    }
}

pub(crate) async fn traces() -> Markup {
    html! {
        code $"w-full font-mono text-xs leading-snug" get="/admin/traces" trigger="load delay:3s" into="this" swap-full
            {(PreEscaped(LOGS.traces()))}
    }
}
