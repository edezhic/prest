use prest::*;

include_as!(Examples from "../" only "*.md");

pub fn shared() -> Router {
    let mut router = Router::new().route("/", get(into_html(include_bytes!("../../README.md"))));
    let mut menu = vec![];
    for path in Examples::iter() {
        let url = format!("/{}", &path.trim_end_matches("/README.md"));
        router = router.route(&url, get(into_html(&(Examples::get(&path).unwrap().data))));
        menu.push((url.clone(), url.replace("/", "").replace("-", " ")));
    }
    router.route_layer(HTMXify::wrap(move |content| page(content, &menu)))
}

fn page(content: Markup, menu: &Vec<(String, String)>) -> Markup {
    html!((DOCTYPE) html data-theme="dark" {
        (Head::default_pwa().title("Prest Blog").css("/styles.css"))
        body hx-boost="true" hx-swap="innerHTML transition:true show:window:top" hx-target="main" _="on click remove .visible from #menu-bar" {
            header."top container" {
                nav style="position:relative; padding:0 16px"{
                    ul { h3."logo"{ li { a href="/" {"prest"}}}}
                    ul {
                        //li { a href="https://docs.rs/prest" target="_blank" {(PreEscaped(include_str!("assets/docs.svg")))}}
                        li { a href="https://github.com/edezhic/prest" target="_blank" {(PreEscaped(include_str!("assets/github.svg")))}}
                        li _="on click toggle .visible on #menu-bar then halt the event" { small{"examples  "}(PreEscaped(include_str!("assets/menu.svg")))}
                        aside #"menu-bar" { ul { @for (url, name) in menu {
                                li { a href={(url)} {small{(name)}}} hr{}
                        }}}
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

fn into_html(data: &[u8]) -> String {
    use markdown::{to_html_with_options, Options};
    let md = std::str::from_utf8(data).unwrap().to_owned();
    #[cfg(debug_assertions)]
    let md = md.replace("https://prest.blog", "http://localhost");
    to_html_with_options(&md, &Options::gfm()).unwrap()
}

#[cfg(feature = "host")]
#[tokio::main(flavor = "current_thread")]
pub async fn main() {
    include_build_output_as!(Dist);
    let host_svc = shared()
        .embed(Dist)
        .route("/styles.css", get(Css(include_str!("assets/styles.css"))))
        .route("/favicon.ico", get(Favicon(include_bytes!("assets/favicon.ico").as_slice())));
    serve(host_svc, Default::default()).await
}

#[cfg(feature = "sw")]
#[wasm_bindgen]
pub async fn handle_fetch(sw: ServiceWorkerGlobalScope, fe: FetchEvent) {
    serve(shared(), sw, fe).await
}
