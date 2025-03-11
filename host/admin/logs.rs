use chrono::NaiveDate;
use host::{admin::LOADER_SVG, TRACES_DATE_FORMAT};

use crate::{host::LOGS, *};

pub(crate) async fn info_explorer() -> Markup {
    html! {
        $"w-full"{
            $"font-bold text-lg" {"Latest info"}
            $"font-mono text-[0.5rem] md:text-sm leading-snug" {
                $"w-8 mx-auto" get="/admin/latest_info/0" trigger="revealed" swap-this {(LOADER_SVG)}
            }
        }
    }
}

pub(crate) async fn info(Path(offset): Path<usize>) -> Markup {
    const PER_PAGE: usize = 20;
    let logs: Vec<_> = LOGS
        .latest_info(offset, PER_PAGE)
        .into_iter()
        .map(|log| PreEscaped(log))
        .collect();

    let maybe_more = logs.len() > 0;

    html! {
        @for log in logs {p style="margin:0 !important"{(log)}}
        @if maybe_more {
            $"w-8 mx-auto"
                get={"/admin/latest_info/"(offset + PER_PAGE)}
                trigger="revealed"
                target="this"
                swap="outerHTML transition:false"
                {(LOADER_SVG)}
        }
    }
}

pub(crate) async fn traces_explorer() -> Markup {
    let today = Utc::now().format(TRACES_DATE_FORMAT);
    let mut available_dates = LOGS.recorded_traces_dates();
    available_dates.sort_by(|a, b| b.cmp(a));

    html! {
        a _=(format!("on load call loadTraces('{today}') then remove me")) {}
        select $"bg-stone-900 accent-stone-600 px-2 py-1" _="on every change call loadTraces(event.target.value)" {
            @for date in available_dates {
                option value=(date) {(date)}
            }
        }
        #"traces-container" $"font-mono" {}
    }
}

pub(crate) async fn traces(Path(date): Path<NaiveDate>) -> impl IntoResponse {
    LOGS.traces(date)
}
