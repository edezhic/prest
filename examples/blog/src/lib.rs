use prest::*;

mod content;
use content::{ExampleCategory::*, EXAMPLES, INTERNALS, PREST_VERSION, README};

pub fn routes() -> Router {
    let mut router = route("/", get(README.clone())).route("/internals", get(INTERNALS.clone()));
    for readme in EXAMPLES.iter() {
        router = router.route(&readme.url, get(readme.content.as_str()));
    }
    router.wrap_non_htmx(page)
}

async fn page(content: Markup) -> Markup {
    let dbs = EXAMPLES.iter().filter(|r| r.category == Database);
    let todos = EXAMPLES.iter().filter(|r| r.category == Todo);
    let others = EXAMPLES.iter().filter(|r| r.category == Other);
    html!((DOCTYPE) html data-theme="dark" {
        (Head::with_title("Prest Blog").style(CODE_STYLES).css("https://unpkg.com/prismjs@1.29.0/themes/prism-tomorrow.min.css"))
        body."max-w-screen-md lg:max-w-screen-lg container md:mx-auto"
            hx-boost="true" hx-swap="innerHTML transition:true show:window:top" hx-target="main" {
            nav."navbar bg-base-200 shadow-lg rounded-box my-4"{
                ."navbar-start md:gap-2" {
                    a."btn btn-ghost btn-circle" href="https://docs.rs/prest" target="_blank" {(PreEscaped(include_str!("../icons/docs.svg")))}
                    a."btn btn-ghost btn-circle" href="https://github.com/edezhic/prest" target="_blank" {(PreEscaped(include_str!("../icons/github.svg")))}
                }
                ."navbar-center" { a."btn btn-ghost" href="/" {"PREST"} }
                ."navbar-end"{."dropdown dropdown-bottom dropdown-end"{
                    @if is_pwa() {
                        ."indicator mr-4" hx-get="/sw/health" hx-target="this" hx-trigger="load delay:3s" hx-swap="none"
                            hx-on--after-request="
                                const b = document.querySelector('#sw-badge'); 
                                const s = event.detail.successful ? 'badge-success' : 'badge-error'; 
                                b.classList.replace('badge-warning', s)" 
                            {
                            #"sw-badge" ."indicator-item badge badge-warning"{}
                            ."font-bold" {"PWA"}
                        }
                    }
                    ."btn btn-ghost btn-circle" tabindex="0" role="button" {
                        svg."h-5 w-5" style="transform: scale(-1,1)" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                            path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h7" {}
                        }
                    }
                    ul."menu menu-md dropdown-content mt-3 z-10 p-2 bg-base-300 shadow-xl rounded-box w-52" tabindex="0" {
                        li { a href="/internals" {"internals"}}
                        li{ h2."menu-title"{"tutorials"}
                            ul{@for r in todos {
                                li { a href={(r.url)} {(r.label)}}
                        }}}
                        li{ h2."menu-title"{"databases"}
                            ul{@for r in dbs {
                                li { a href={(r.url)} {(r.label)}}
                        }}}
                        li{ h2."menu-title"{"others"}
                            ul{@for r in others {
                                li { a href={(r.url)} {(r.label)}}
                        }}}
                        li { a href="/about" {"about"}}
                    }
                }}
            }
            main."view-transition mx-auto p-4 prose lg:prose-xl lg:w-[1024px] lg:max-w-[1024px] [overflow-wrap:anywhere]"
                hx-history-elt hx-on--before-swap="document.activeElement.blur()"
                hx-on--after-swap=(format!("Prism.highlightAll(); {LINKS_JS}"))
                {(content)}
            script {(PreEscaped(LINKS_JS))}
            ."menu menu-horizontal w-full items-center justify-center bg-base-200 rounded-box mb-4 mx-auto"{
                ."font-mono" {"v"(*PREST_VERSION)}
                ."ml-4 mr-2" {"made by Egor Dezhic"}
                a."btn btn-ghost btn-circle" href="https://twitter.com/eDezhic" target="_blank" {(PreEscaped(include_str!("../icons/twitter.svg")))}
                a."btn btn-ghost btn-circle" href="https://edezhic.medium.com" target="_blank" {(PreEscaped(include_str!("../icons/medium.svg")))}
                a."btn btn-ghost btn-circle" href="mailto:edezhic@gmail.com" target="_blank" {(PreEscaped(include_str!("../icons/email.svg")))}
            }
            (Scripts::default()
                .include("https://unpkg.com/prismjs@1.29.0/components/prism-core.min.js")
                .include("https://unpkg.com/prismjs@1.29.0/plugins/autoloader/prism-autoloader.min.js")
            )
        }
    })
}

const LINKS_JS: &str = "document.querySelectorAll('main a').forEach(el => !el.href.includes('prest') && !el.href.includes('localhost') && el.setAttribute('target', '_blank'))";
const CODE_STYLES: PreEscaped<&str> = PreEscaped(
    r#"
code {
    font-size: 12px !important;
}
@media screen and (min-width: 1024px) {
    code {
        font-size: 15px !important;
    }
}
code .table {
    display: inherit;
    font-size: inherit;
}
"#,
);

#[cfg(sw)]
#[wasm_bindgen(start)]
pub fn main() {
    routes().handle_fetch_events()
}
