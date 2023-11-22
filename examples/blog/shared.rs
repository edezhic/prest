use prest::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn handle_fetch(sw: ServiceWorkerGlobalScope, fe: FetchEvent) {
    serve(routes(), sw, fe).await
}

pub fn routes() -> Router {
    let mut router = Router::new().route("/", get(md_to_html(include_bytes!("../../README.md"))));
    let mut links = vec![];
    
    embed_as!(Examples from "../" only "*.md");
    for path in Examples::iter() {
        let url = if path.starts_with("blog") {
            "/about".to_owned()
        } else {
            format!("/{}", &path.trim_end_matches("/README.md"))
        };
        router = router.route(&url, get(md_to_html(&(Examples::get(&path).unwrap().data))));

        let label = url.replace("/", "").replace("-", " ");
        links.push((url, label));
    }
    
    router.wrap_non_htmx(move |content| page(content, &links))
}

fn md_to_html(data: &[u8]) -> String {
    use markdown::{to_html_with_options, Options};
    let md = std::str::from_utf8(data).unwrap().to_owned();
    #[cfg(debug_assertions)]
    let md = md.replace("https://prest.blog", "http://localhost");
    to_html_with_options(&md, &Options::gfm()).unwrap()
}

fn page(content: Markup, links: &Vec<(String, String)>) -> Markup {
    let hello = links.iter().filter(|(_, label)| label.starts_with("hello"));
    let into = links.iter().filter(|(_, label)| label.starts_with("into"));
    let with = links.iter().filter(|(_, label)| label.starts_with("with"));
    html!((DOCTYPE) html data-theme="dark" {
        (Head::example().title("Prest Blog").css("/styles.css").release_pwa())
        body hx-boost="true" hx-swap="innerHTML transition:true show:window:top" hx-target="main" _="on click remove .visible from #examples-menu" {
            header."top container" {
                nav style="position:relative; padding:0 16px"{
                    ul { h3."logo"{ li { a href="/" {"PREST"}}}}
                    ul {
                        li { a href="https://docs.rs/prest" target="_blank" {(PreEscaped(include_str!("assets/docs.svg")))}}
                        li { a href="https://github.com/edezhic/prest" target="_blank" {(PreEscaped(include_str!("assets/github.svg")))}}
                        li #"examples-btn" _="on click toggle .visible on #examples-menu then halt the event" {
                            "examples"(PreEscaped(include_str!("assets/menu.svg")))
                        }
                        aside #"examples-menu" { ul {
                            li { h6{"step by step"} }
                            @for (url, label) in hello {
                                li { a href={(url)} {small{(label.trim_start_matches("hello "))}}}
                            }
                            hr{}
                            li { h6{"into"} }
                            @for (url, label) in into {
                                li { a href={(url)} {small{(label.trim_start_matches("into "))}}}
                            }
                            hr{}
                            li { h6{"with"} }
                            @for (url, label) in with {
                                li { a href={(url)} {small{(label.trim_start_matches("with "))}}}
                            }
                            hr{}
                            li { a href="/about" {small{"about"}}}
                        }}
                    }
                }
            }
            main."container slide-transition" style="padding:16px" {(content)}
            footer."container" style="padding:24px" {
                small{i{"Made by Egor Dezhic"
                    a href="https://twitter.com/eDezhic" target="_blank" {(PreEscaped(include_str!("assets/twitter.svg")))}
                    a href="https://edezhic.medium.com" target="_blank" {(PreEscaped(include_str!("assets/medium.svg")))}
                    a href="mailto:edezhic@gmail.com" target="_blank" {(PreEscaped(include_str!("assets/email.svg")))}
                }}
            }
            (Scripts::default_pwa())
        }
    })
}
