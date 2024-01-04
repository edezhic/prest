use markdown::{to_html_with_options, Options};
use prest::*;

embed_as!(ExamplesCode from "../" except "*.md");
embed_as!(ExamplesDocs from "../" only "*.md");

state!(READMES: Vec<(String, String, String)> = {
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
    let mut router = route("/", get(home_doc));
    for (doc_path, url, _) in READMES.iter() {
        let raw_doc = ExamplesDocs::get_content(&doc_path).unwrap();
        let processed = preprocess_md(raw_doc, &doc_path);
        let doc_html = md_to_html(&processed);
        router = router.route(&url, get(Html(doc_html)));
    }
    router.wrap_non_htmx(page)
}

fn preprocess_md(raw_doc: String, doc_path: &str) -> String {
    let mut processed = String::new();
    for line in raw_doc.lines() {
        // lines like {path} are converted into the contents of path
        if line.starts_with("{") && line.ends_with("}") {
            let inline_file = line.replace(['{', '}'], "");
            let inline_path = format!("{}/{inline_file}", doc_path.trim_end_matches("/README.md"));
            let code = match ExamplesCode::get_content(&inline_path) {
                Some(code) => code,
                None => panic!("Not found {inline_path} mentioned in {doc_path}"),
            };
            let code_type = match &inline_file {
                f if f.ends_with(".rs") => "rust",
                f if f.ends_with(".toml") => "toml",
                f if f.ends_with(".css") => "css",
                f if f.ends_with(".scss") => "scss",
                f if f.ends_with(".html") => "html",
                f if f.ends_with(".sql") => "sql",
                f if f.ends_with(".ts") => "typescript",
                _ => "",
            };
            processed += &format!("`/{inline_file}`\n");
            processed += &format!("\n```{code_type}\n{}\n```\n", code.trim_end());
        } else {
            processed += &format!("{line}\n");
        }
    }
    processed
}

fn md_to_html(md: &str) -> String {
    #[cfg(debug_assertions)]
    let md = md.replace("https://prest.blog", "http://localhost");
    to_html_with_options(&md, &Options::gfm()).unwrap()
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
        body."max-w-screen-md lg:max-w-screen-lg container md:mx-auto" 
            hx-boost="true" hx-swap="innerHTML transition:true show:window:top" hx-target="main" {
            nav."navbar bg-base-200 shadow-lg rounded-box my-4"{
                ."navbar-start md:gap-2" {
                    a."btn btn-ghost btn-circle" href="https://docs.rs/prest" target="_blank" {(PreEscaped(include_str!("assets/docs.svg")))}
                    a."btn btn-ghost btn-circle" href="https://github.com/edezhic/prest" target="_blank" {(PreEscaped(include_str!("assets/github.svg")))}
                }
                ."navbar-center" { a."btn btn-ghost" href="/" {"PREST"} }
                ."navbar-end"{."dropdown dropdown-bottom dropdown-end"{
                    ."indicator mr-4" hx-get="/sw/health" hx-target="this" hx-trigger="load delay:3s" hx-swap="none" 
                        hx-on--after-request="
                            const b = document.querySelector('#sw-badge'); 
                            const s = event.detail.successful ? 'badge-success' : 'badge-error'; 
                            b.classList.replace('badge-warning', s)" 
                        {
                        #"sw-badge" ."indicator-item badge badge-warning"{}
                        ."font-bold" {"PWA"}
                    }
                    ."btn btn-ghost btn-circle" tabindex="0" role="button" {
                        svg."h-5 w-5" style="transform: scale(-1,1)" fill="none" viewBox="0 0 24 24" stroke="currentColor" { 
                            path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h7" {}
                        }
                    }
                    ul."menu menu-sm dropdown-content mt-3 z-10 p-2 bg-base-300 shadow-xl rounded-box w-52" tabindex="0" {
                        li{ h2."menu-title"{"apps"}
                            ul{li{a href="/app-todo" {small{"todo"}}}}
                        }
                        li{ h2."menu-title"{"step by step"}
                            ul {@for (_, url, label) in hello {
                                li { a href={(url)} {small{(label.trim_start_matches("hello "))}}}
                        }}}
                        li{ h2."menu-title"{"into"}
                            ul {@for (_, url, label) in into {
                                li { a href={(url)} {small{(label.trim_start_matches("into "))}}}
                        }}}
                        li{ h2."menu-title"{"with"}
                            ul{@for (_, url, label) in with {
                                li { a href={(url)} {small{(label.trim_start_matches("with "))}}}
                        }}}
                        li { a href="/about" {small{"about"}}}
                    }
                }}
            }
            main."view-transition mx-auto p-4 prose lg:prose-xl lg:w-[1024px] lg:max-w-[1024px]" 
                hx-history-elt hx-on--before-swap="document.activeElement.blur()" hx-on--after-swap="Prism.highlightAll()" {(content)}
            ."menu menu-horizontal w-full items-center justify-center bg-base-200 rounded-box mb-4 mx-auto"{
                a."mr-4" {"Made by Egor Dezhic"}
                a."btn btn-ghost btn-circle" href="https://twitter.com/eDezhic" target="_blank" {(PreEscaped(include_str!("assets/twitter.svg")))}
                a."btn btn-ghost btn-circle" href="https://edezhic.medium.com" target="_blank" {(PreEscaped(include_str!("assets/medium.svg")))}
                a."btn btn-ghost btn-circle" href="mailto:edezhic@gmail.com" target="_blank" {(PreEscaped(include_str!("assets/email.svg")))}
            }
            (Scripts::default()
                .include("https://unpkg.com/prismjs@1.29.0/components/prism-core.min.js")
                .include("https://unpkg.com/prismjs@1.29.0/plugins/autoloader/prism-autoloader.min.js")
            )
        }
    })
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main() {
    routes().serve()
}
