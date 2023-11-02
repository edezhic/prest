use prest::*;

include_as!(Notes from "../../" only "*.md");
impl Notes {
    async fn get_html(path: Cow<'_, str>) -> String {
        use markdown::{to_html_with_options, Options};
        let file = Self::get(&path).unwrap();
        let md = std::str::from_utf8(&file.data).unwrap();
        to_html_with_options(md, &Options::gfm()).unwrap()
    }
}

#[derive(Clone)]
enum MenuItemKind {
    Example,
    Module,
    Doc,
}
#[derive(Clone)]
struct MenuItem {
    pub kind: MenuItemKind,
    pub name: String,
    pub path: String,
}

pub fn shared() -> Router {
    let mut router = Router::new();
    let mut menu = vec![];
    for file_path in Notes::iter() {
        let (name, kind) = match file_path.as_ref() {
            "README.md" => ("", None),
            p if p.starts_with("docs/") => (
                p.trim_start_matches("docs/").trim_end_matches(".md"),
                Some(MenuItemKind::Doc),
            ),
            p if p.starts_with("examples/") => (
                extract_last_dir_name(p).unwrap(),
                Some(MenuItemKind::Example),
            ),
            p => (
                extract_last_dir_name(p).unwrap(),
                Some(MenuItemKind::Module),
            ),
        };
        let url_path = "/".to_owned() + name;
        if let Some(kind) = kind {
            menu.push(MenuItem {
                kind,
                name: name.to_owned(),
                path: url_path.clone(),
            })
        }
        router = router.route(&url_path, get(move || Notes::get_html(file_path)));
    }
    router.route_layer(HTMXify::wrap(move |content| page(&menu, content)))
}

fn page(menu: &Vec<MenuItem>, content: Markup) -> Markup {
    html!((DOCTYPE) html data-theme="dark" {
        (Head::pwa().title("Prest Blog").css("/styles.css"))
        body hx-boost="true" hx-swap="innerHTML transition:true show:window:top" hx-target="main" {
            header."top container" {
                nav {
                    ul { li { a {
                        (PreEscaped(include_str!("assets/menu.svg")))
                    }}}
                    ul { h3."logo"{ li { a href="/" {"prest"} } } }
                    ul {
                        @for item in menu { @if let MenuItemKind::Doc = item.kind {
                            li { a href={(item.path)} {(item.name)} }
                        }}
                        li { a."icon" href="https://github.com/edezhic/prest" target="_blank" {(PreEscaped(include_str!("assets/github.svg")))}}
                    }
                }
            }
            main."container slide-transition" {(content)}
            footer."container" styles="padding: 24px;" {
                small{i{"Made by Egor Dezhic"
                    a."icon" href="https://twitter.com/eDezhic" target="_blank" {(PreEscaped(include_str!("assets/twitter.svg")))}
                    a."icon" href="https://edezhic.medium.com" target="_blank" {(PreEscaped(include_str!("assets/medium.svg")))}
                    a."icon" href="mailto:edezhic@gmail.com" target="_blank" {(PreEscaped(include_str!("assets/email.svg")))}
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

fn extract_last_dir_name(p: &str) -> Option<&str> {
    std::path::Path::new(p)
        .parent()?
        .components()
        .last()?
        .as_os_str()
        .to_str()
}
