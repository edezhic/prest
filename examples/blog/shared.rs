use markdown::{to_html_with_options, Options};
use prest::*;

embed_as!(ExamplesCode from "../" only "*.rs", "*.toml");
embed_as!(ExamplesDocs from "../" only "*.md");
static READMES: Lazy<Vec<(String, String, String)>> = Lazy::new(|| {
    let mut examples = vec![];
    for path in ExamplesDocs::iter() {
        let path = path.to_string();
        let url = if path.starts_with("blog") {
            "/about".to_owned()
        } else {
            format!("/{}", path.trim_end_matches("/README.md"))
        };
        let label = url.replace("/", "").replace("-", " ");
        examples.push((path, url, label));
    }
    examples
});

pub fn routes() -> Router {
    let home_doc = md_to_html(include_str!("../../README.md"));
    let mut router = Router::new().route("/", get(home_doc));
    for (path, url, _) in READMES.iter() {
        router = router.route(&url, get(Html(gen_doc(path))));
    }
    router.wrap_non_htmx(page)
}

fn gen_doc(doc_path: &str) -> String {
    let readme = ExamplesDocs::get_content(&doc_path).unwrap();
    let mut processed = String::new();
    for line in readme.lines() {
        // lines like {path} are converted into the contents of path
        if line.starts_with("{") && line.ends_with("}") {
            let inline_file = line.replace(['{', '}'], "");
            let inline_path = format!("{}/{inline_file}", doc_path.trim_end_matches("/README.md"));
            let code = ExamplesCode::get_content(&inline_path).unwrap();
            processed += &format!("\n```rust\n{code}\n```\n");
        } else {
            processed += line;
            processed += "\n";
        }
    }
    md_to_html(&processed)
}

fn md_to_html(str: &str) -> String {
    #[cfg(debug_assertions)]
    let str = str.replace("https://prest.blog", "http://localhost");
    to_html_with_options(&str, &Options::gfm()).unwrap()
}

async fn page(content: Markup) -> Markup {
    let hello = READMES
        .iter()
        .filter(|(_, _, label)| label.starts_with("hello"));
    let into = READMES
        .iter()
        .filter(|(_, _, label)| label.starts_with("into"));
    let with = READMES
        .iter()
        .filter(|(_, _, label)| label.starts_with("with"));
    html!((DOCTYPE) html data-theme="dark" {
        (Head::example("Prest Blog").css("/styles.css").css("https://unpkg.com/prismjs@1.29.0/themes/prism-tomorrow.min.css"))
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
                            @for (_, url, label) in hello {
                                li { a href={(url)} {small{(label.trim_start_matches("hello "))}}}
                            }
                            hr{}
                            li { h6{"into"} }
                            @for (_, url, label) in into {
                                li { a href={(url)} {small{(label.trim_start_matches("into "))}}}
                            }
                            hr{}
                            li { h6{"with"} }
                            @for (_, url, label) in with {
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
            (Scripts::default()
                .include("https://unpkg.com/prismjs@1.29.0/components/prism-core.min.js")
                .include("https://unpkg.com/prismjs@1.29.0/plugins/autoloader/prism-autoloader.min.js")
                .inline("document.addEventListener('htmx:afterSwap', () => Prism.highlightAll())")    
            )
        }
    })
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn handle_fetch(sw: ServiceWorkerGlobalScope, fe: FetchEvent) {
    serve(routes(), sw, fe).await
}
