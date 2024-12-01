use crate::*;

mod db_editor;
mod logs;
mod remote;
mod routes_stats;
mod schedule_stats;
mod system_stats;

pub(crate) fn routes() -> Router {
    route("/", get(system_stats::full))
        // .route("/system", get())
        .route("/logs", get(logs::latest_info))
        .route("/traces", get(logs::traces))
        .route("/schedule_stats", get(schedule_stats::full))
        .route("/analytics", get(routes_stats::full))
        .nest("/remote", remote::routes())
        .nest("/db", db_editor::db_routes())
        .wrap_non_htmx(into_page)
}

async fn into_page(content: Markup) -> impl IntoResponse {
    let page = html! {(DOCTYPE) html $"bg-stone-800 font-sans text-gray-300" {
        (Head::with_title("Prest Admin"))
        body $"max-w-screen-md lg:max-w-screen-lg md:mx-auto" {
            nav $"bg-stone-900 my-4 p-5 shadow-lg rounded-full items-center flex gap-6 w-min mx-auto" {
                a href="/" boost="false" {(home_svg())}
                button get="/admin" into="main" {(system_svg())}
                button get="/admin/analytics" into="main" {(analytics_svg())}
                button get="/admin/logs" into="main" {(logs_svg())}
                @if DB_SCHEMA.tables().len() > 0 {
                    button get="/admin/db" into="main" {(db_svg())}
                }
            }
            main $"opacity-80 mx-auto p-4 gap-3 flex flex-col text-sm lg:text-base leading-loose" {
                (content)
            }
            (Scripts::default())
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

fn system_svg() -> Markup {
    html!(
        svg $"w-6" viewBox="0 0 12 12" enable-background="new 0 0 12 12" {
            path d="M10.25,6c0-0.1243286-0.0261841-0.241333-0.0366211-0.362915l1.6077881-1.5545654l-1.25-2.1650391  c0,0-1.2674561,0.3625488-2.1323853,0.6099854c-0.2034912-0.1431885-0.421875-0.2639771-0.6494751-0.3701782L7.25,0h-2.5  c0,0-0.3214111,1.2857666-0.5393066,2.1572876C3.9830933,2.2634888,3.7647095,2.3842773,3.5612183,2.5274658L1.428833,1.9174805  l-1.25,2.1650391c0,0,0.9641113,0.9321899,1.6077881,1.5545654C1.7761841,5.758667,1.75,5.8756714,1.75,6  s0.0261841,0.241333,0.0366211,0.362915L0.178833,7.9174805l1.25,2.1650391l2.1323853-0.6099854  c0.2034912,0.1432495,0.421875,0.2639771,0.6494751,0.3701782L4.75,12h2.5l0.5393066-2.1572876  c0.2276001-0.1062012,0.4459839-0.2269287,0.6494751-0.3701782l2.1323853,0.6099854l1.25-2.1650391L10.2133789,6.362915  C10.2238159,6.241333,10.25,6.1243286,10.25,6z M6,7.5C5.1715698,7.5,4.5,6.8284302,4.5,6S5.1715698,4.5,6,4.5S7.5,5.1715698,7.5,6  S6.8284302,7.5,6,7.5z" fill="currentColor"{}
        }
    )
}

fn analytics_svg() -> Markup {
    html!(
        svg $"w-6" viewBox="0 0 1024 1024" fill="currentColor" {
            path d="M521.58 516.763v-472.816c250.725 22.642 450.175 222.092 472.817 472.817h-472.816zM918.229 593.091h-435.436c-21.963 0-39.769-17.805-39.769-39.769 0 0 0 0 0 0v-435.463c-222.914 20.121-397.682 207.273-397.682 435.436 0 241.605 195.898 437.452 437.451 437.451 228.163 0 415.339-174.715 435.436-397.657z" {}
        }
    )
}

fn logs_svg() -> Markup {
    html!(
        svg $"w-6" viewBox="0 0 16 16" fill="none" {
            g fill="currentColor" {
                path d="M5.314 1.256a.75.75 0 01-.07 1.058L3.889 3.5l1.355 1.186a.75.75 0 11-.988 1.128l-2-1.75a.75.75 0 010-1.128l2-1.75a.75.75 0 011.058.07zM7.186 1.256a.75.75 0 00.07 1.058L8.611 3.5 7.256 4.686a.75.75 0 10.988 1.128l2-1.75a.75.75 0 000-1.128l-2-1.75a.75.75 0 00-1.058.07zM2.75 7.5a.75.75 0 000 1.5h10.5a.75.75 0 000-1.5H2.75zM2 11.25a.75.75 0 01.75-.75h10.5a.75.75 0 010 1.5H2.75a.75.75 0 01-.75-.75zM2.75 13.5a.75.75 0 000 1.5h6.5a.75.75 0 000-1.5h-6.5z" {}
            }
        }
    )
}

fn db_svg() -> Markup {
    html!(
        svg $"w-6" viewBox="0 0 24 24" fill="none" {
            path d="M20 18C20 20.2091 16.4183 22 12 22C7.58172 22 4 20.2091 4 18V13.974C4.50221 14.5906 5.21495 15.1029 6.00774 15.4992C7.58004 16.2854 9.69967 16.75 12 16.75C14.3003 16.75 16.42 16.2854 17.9923 15.4992C18.7851 15.1029 19.4978 14.5906 20 13.974V18Z" fill="currentColor" {}
            path d="M12 10.75C14.3003 10.75 16.42 10.2854 17.9923 9.49925C18.7851 9.10285 19.4978 8.59059 20 7.97397V12C20 12.5 18.2143 13.5911 17.3214 14.1576C15.9983 14.8192 14.118 15.25 12 15.25C9.88205 15.25 8.00168 14.8192 6.67856 14.1576C5.5 13.5683 4 12.5 4 12V7.97397C4.50221 8.59059 5.21495 9.10285 6.00774 9.49925C7.58004 10.2854 9.69967 10.75 12 10.75Z" fill="currentColor" {}
            path d="M17.3214 8.15761C15.9983 8.81917 14.118 9.25 12 9.25C9.88205 9.25 8.00168 8.81917 6.67856 8.15761C6.16384 7.95596 5.00637 7.31492 4.2015 6.27935C4.06454 6.10313 4.00576 5.87853 4.03988 5.65798C4.06283 5.50969 4.0948 5.35695 4.13578 5.26226C4.82815 3.40554 8.0858 2 12 2C15.9142 2 19.1718 3.40554 19.8642 5.26226C19.9052 5.35695 19.9372 5.50969 19.9601 5.65798C19.9942 5.87853 19.9355 6.10313 19.7985 6.27935C18.9936 7.31492 17.8362 7.95596 17.3214 8.15761Z" fill="currentColor" {}
        }
    )
}
