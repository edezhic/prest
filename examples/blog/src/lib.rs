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

    html!((DOCTYPE) html $"bg-gray-800 font-sans text-[#bbc4d4]" {
        (Head::with_title("Prest Blog").style(STYLES))

        body $"max-w-screen-md lg:max-w-screen-lg md:mx-auto"
            hx-boost="true" hx-swap="innerHTML transition:true show:window:top" hx-target="main" {
            nav $"bg-gray-900 my-4 p-5 shadow-lg rounded-full grid grid-cols-3 items-center" {
                $"flex gap-4" {
                    a $"hover:text-white mr-4" href="https://docs.rs/prest" target="_blank" {(PreEscaped(include_str!("../icons/docs.svg")))}
                    a $"hover:text-white" href="https://github.com/edezhic/prest" target="_blank" {(PreEscaped(include_str!("../icons/github.svg")))}
                }
                a $"font-bold text-center hover:text-white" href="/" {"PREST"}
                $"flex justify-end" {
                    @if is_pwa() {
                        $"mr-4" hx-get="/sw/health" hx-target="this" hx-trigger="load delay:3s" hx-swap="none"
                            hx-on--after-request="
                                const b = document.querySelector('#sw-badge'); 
                                const s = event.detail.successful ? 'badge-success' : 'badge-error'; 
                                b.classList.replace('badge-warning', s)" 
                            {
                            #"sw-badge" {}
                            $"font-bold" {"SW"}
                        }
                    }
                    div tabindex="0" role="button" $"hover:text-white" {
                        svg $"h-5 w-5 scale-[-1,1]" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                            path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h7" {}
                        }
                    }
                    ul $"bg-gray-900 mt-3 z-10 p-2 shadow-xl rounded-full w-52 hidden" tabindex="0" {
                        li { a href="/internals" {"internals"}}
                        li{ h2 {"tutorials"}
                            ul{@for r in todos {
                                li { a href={(r.url)} {(r.label)}}
                        }}}
                        li{ h2 {"databases"}
                            ul{@for r in dbs {
                                li { a href={(r.url)} {(r.label)}}
                        }}}
                        li{ h2 {"others"}
                            ul{@for r in others {
                                li { a href={(r.url)} {(r.label)}}
                        }}}
                        li { a href="/about" {"about"}}
                    }
                }
            }
            main $"opacity-80 mx-auto p-4 gap-3 flex flex-col text-sm lg:text-base leading-loose"
                hx-history-elt hx-on--before-swap="document.activeElement.blur()"
                hx-on--after-swap=(format!("Prism.highlightAll(); {LINKS_JS}"))
                {(content)}
            style{""}
            script {(PreEscaped(LINKS_JS))}
            $"flex items-center justify-evenly p-4 w-full bg-gray-900 rounded-full mb-4 mx-auto text-xs lg:text-base" {
                $"font-mono" {"v"(*PREST_VERSION)}
                $"" {"made by Egor Dezhic"}
                $"flex gap-3"{
                    a href="https://twitter.com/eDezhic" target="_blank" {(PreEscaped(include_str!("../icons/twitter.svg")))}
                    a href="https://edezhic.medium.com" target="_blank" {(PreEscaped(include_str!("../icons/medium.svg")))}
                    a href="mailto:edezhic@gmail.com" target="_blank" {(PreEscaped(include_str!("../icons/email.svg")))}
                }
            }
            (Scripts::default()
                .css("https://unpkg.com/prismjs@1.29.0/themes/prism-tomorrow.min.css")
                .include("https://unpkg.com/prismjs@1.29.0/components/prism-core.min.js")
                .include("https://unpkg.com/prismjs@1.29.0/plugins/autoloader/prism-autoloader.min.js")
            )
        }
    })
}

const LINKS_JS: &str = "document.querySelectorAll('main a').forEach(el => !el.href.includes('prest') && !el.href.includes('localhost') && el.setAttribute('target', '_blank'))";
const STYLES: PreEscaped<&str> = PreEscaped(
    r#"
main a { text-decoration: underline } 
main h3 { font-size: 2em } 
main ul { list-style: circle }
code { font-size: 12px !important }
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
