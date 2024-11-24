use prest::*;

mod content;
pub use content::{ExampleCategory::*, EXAMPLES, INTERNALS, PREST_VERSION, README, RUST};

pub fn routes() -> Router {
    let mut router = route("/", get(README.clone()))
        .route("/internals", get(INTERNALS.clone()))
        .route("/rust", get(RUST.clone()));
    for readme in EXAMPLES.iter() {
        router = router.route(&readme.url, get(readme.content.as_str()));
    }
    router.wrap_non_htmx(page)
}

async fn page(content: Markup) -> Markup {
    let dbs = EXAMPLES.iter().filter(|r| r.category == Database);
    let todos = EXAMPLES.iter().filter(|r| r.category == Todo);
    let others = EXAMPLES.iter().filter(|r| r.category == Other);

    html!((DOCTYPE) html $"bg-stone-800 font-sans text-[#bbc4d4]" _="on click remove .open from #menu" {
        (Head::with_title("Prest Blog"))

        body $"max-w-screen-md lg:max-w-screen-lg md:mx-auto"
            hx-boost="true" hx-swap="innerHTML transition:true show:window:top" into="main" {
            nav $"bg-stone-900 my-4 p-5 shadow-lg rounded-full grid grid-cols-3 items-center" {
                $"flex gap-6" {
                    a $"hover:text-white" href="https://github.com/edezhic/prest" {(include_html!("../icons/github.svg"))}
                    a $"hover:text-white" href="https://docs.rs/prest" {(include_html!("../icons/docs.svg"))}
                }

                a $"font-bold text-center hover:text-white" href="/" {"PREST"}

                $"flex justify-end" {
                    @if is_pwa() {
                        div #"sw-badge" $"mr-6 font-bold text-sm" get="/sw/health" into="this" trigger="every 3s delay:3s" swap-none
                        _="on htmx:afterRequest
                            if event.detail.successful set #sw-badge.style.color to '#059669'
                            else set #sw-badge.style.color to '#991b1b'"
                        {"SW"}
                    }

                    $"hover:text-white" _="on click add .open to #menu halt" {
                        svg $"h-5 w-5 scale-[-1,1]" fill="none" viewBox="0 0 24 24" stroke="currentColor" {
                            path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h7" {}
                        }
                    }

                    div #"menu" $"absolute bg-stone-950 z-10 top-8 px-4 truncate shadow-xl rounded-xl w-52" {
                        style {"
                            #menu { max-height: 0px } #menu.open { max-height: 1000px } 
                            #menu a { display: flex; align-items: center; padding: 0.25rem 0 0.25rem 0.5rem; border-radius: 1rem; }
                            #menu a:hover { background-color: #292524 }
                        "}
                        $"py-4 flex flex-col gap-2 text-xs" {
                            a href="/rust" {"about rust"}
                            a href="/internals" {"internals"}
                            a href="/about" {"about blog"}
                            $"font-bold text-sm pt-2" {"tutorials"}
                            @for r in todos { a href={(r.url)} {(r.label)}}
                            $"font-bold text-sm pt-2" {"databases"}
                            @for r in dbs { a href={(r.url)} {(r.label)}}
                            $"font-bold text-sm pt-2" {"others"}
                            @for r in others { a href={(r.url)} {(r.label)}}
                        }
                    }
                }
            }

            style {r#"
                main a { text-decoration: underline } 
                main h3 { font-size: 2em } 
                main ul, main ol { list-style: circle }
                code { font-size: 13px !important }
                "#}

            main $"opacity-80 mx-auto p-4 gap-3 flex flex-col text-sm lg:text-base leading-loose"
                hx-history-elt _="on load or htmx:afterSwap call format_content()"
                {(content)}

            $"flex items-center justify-evenly p-4 w-full bg-stone-900 rounded-full mb-4 mx-auto text-xs lg:text-base" {
                $"font-mono" {"v"(*PREST_VERSION)}
                $"text-sm" {"made by Egor Dezhic"}
                $"flex gap-3"{
                    a href="https://twitter.com/eDezhic" {(include_html!("../icons/twitter.svg"))}
                    a href="mailto:edezhic@gmail.com" {(include_html!("../icons/email.svg"))}
                }
            }

            (Scripts::default()
                .css("https://unpkg.com/prismjs@1.29.0/themes/prism-tomorrow.min.css")
                .include("https://unpkg.com/prismjs@1.29.0/components/prism-core.min.js")
                .include("https://unpkg.com/prismjs@1.29.0/plugins/autoloader/prism-autoloader.min.js")
                .hyperscript("
                    def format_content()
                        call Prism.highlightAll()
                        for a in <a /> in <body />
                            if not (a.href contains 'prest.') and not (a.href contains 'localhost')
                                set @target of a to '_blank'
                ")
            )
        }
    })
}

#[cfg(sw)]
#[wasm_bindgen(start)]
pub fn main() {
    routes().handle_fetch_events()
}
