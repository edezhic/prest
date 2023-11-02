use prest::*;

include_as!(Notes from "../../" only "*.md");

async fn note(path: Cow<'_, str>) -> String {
    use markdown::{to_html_with_options, Options};
    let file = Notes::get(&path).unwrap();
    let md = std::str::from_utf8(&file.data).unwrap().to_owned();
    #[cfg(debug_assertions)]
    let md = md.replace("https://prest.blog", "http://localhost");
    to_html_with_options(&md, &Options::gfm()).unwrap()
}

pub fn shared() -> Router {
    let mut router = Router::new().route("/", get(|| note("README.md".into())));
    let (mut docs, mut examples, mut modules) = (vec![], vec![], vec![]);

    for path in Notes::iter() {
        let (name, menu) = match path.as_ref().trim_end_matches(".md") {
            p if p.starts_with("docs/") => (p.trim_start_matches("docs/"), &mut docs),
            p if p.starts_with("examples/") => (last_dir(p), &mut examples),
            p if p.contains("/") => (last_dir(p), &mut modules),
            _ => continue,
        };
        let url = format!("/{name}");
        let display_name = name.replace("-", " ");
        menu.push((display_name, url.clone()));
        router = router.route(&url, get(move || note(path)));
    }
    
    router.route_layer(HTMXify::wrap(move |content| {
        page(&docs, &modules, &examples, content)
    }))
}

fn page(
    docs: &Vec<(String, String)>,
    modules: &Vec<(String, String)>,
    examples: &Vec<(String, String)>,
    content: Markup,
) -> Markup {
    html!((DOCTYPE) html data-theme="dark" {
        (Head::pwa().title("Prest Blog").css("/styles.css"))
        body hx-boost="true" hx-swap="innerHTML transition:true show:window:top" hx-target="main" _="on click remove .visible from #menu-bar" {
            header."top container" {
                nav style="position:relative; padding:0 16px"{
                    ul {
                        li { a href="https://github.com/edezhic/prest" target="_blank" {(PreEscaped(include_str!("assets/github.svg")))}}
                        h3."logo"{ li { a href="/" {"prest"}}}
                    }
                    ul { 
                        @for (name, url) in docs { li { a."contrast" href={(url)} {(name)}}}
                        li { a #"menu-btn" _="on click toggle .visible on #menu-bar then halt the event" {(PreEscaped(include_str!("assets/menu.svg")))}}
                        aside #"menu-bar" { 
                            ul { @for (name, url) in modules { li { a href={(url)} {(name)}}} }
                            hr {}
                            ul { @for (name, url) in examples { li { a href={(url)} {small{(name)}}}} }    
                        }
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
        }
    })
}

#[cfg(feature = "host")]
#[tokio::main(flavor = "current_thread")]
pub async fn main() {
    include_build_output_as!(Dist);
    serve(shared().embed(Dist), Default::default()).await
}

#[cfg(feature = "sw")]
#[wasm_bindgen]
pub async fn handle_fetch(sw: ServiceWorkerGlobalScope, fe: FetchEvent) {
    serve(shared(), sw, fe).await
}

fn last_dir(p: &str) -> &str {
    std::path::Path::new(p)
        .parent()
        .unwrap()
        .components()
        .last()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
}
