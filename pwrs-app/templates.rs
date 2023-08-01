pub fn full_html(content: maud::Markup) -> maud::Markup {
    maud::html!(
        html {
            head {
                title {"PWRS app"}
                link rel="icon" href="/favicon.ico" {}
                link rel="manifest" href="/.webmanifest" {}
                link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Roboto:wght@300;400;500;700&display=swap" {}
                link rel="stylesheet" href="/styles.css" {}
                script src="/include_sw.js" {}
                script src="https://unpkg.com/htmx.org@1.9.0" integrity="sha384-aOxz9UdWG0yBiyrTwPeMibmaoq07/d3a96GCbb9x60f3mOt5zwkjdbcHFnKH8qls" crossorigin="anonymous" {}
                meta
                    name="viewport"
                    content="width=device-width, initial-scale=1.0";
            }
            body {
                main."slide-transition" {(content)}
            }
        }
    )
}   
pub mod external_link {
    pub fn render(text: &str, url: &str) -> maud::Markup {
        let href = format!("https://{}", url);
        maud::html!(a."external-link" href={(href)} target="_blank" {(text)})
    }
}
pub mod home {
    use super::external_link::render as link;
    pub fn render() -> maud::Markup {
        maud::html!(
            h1{b{"p"}"rogressive " b{"w"}"eb " b{"r"}"u" b{"s"}"t " b{"app"}"lication"}
            ."highlight"{
                p{b{"Progressive"}" app that starts as a web page and can bootstrap up to a native one"}
                p{"based on " b{"Web"}" standards such as HTML, CSS, TS/JS, APIs, WASM, WASI et cetera"}
                p{"written in " b{"Rust"}" for safety and speed on a wide range of platforms"}
            }
            
            h2{"Getting started"}
            ."highlight"{
                p{"The only prerequisite is the "(link("official rust toolchain", "rustup.rs"))", but I also recommend "(link("VS Code", "code.visualstudio.com"))" IDE for its built-in Git and TypeScript support with "(link("rust-analyzer", "rust-analyzer.github.io"))" extension for the best dev experience."}
                p{"Now it's time to clone the "(link("pwrsapp repo", "github.com/edezhic/pwrsapp"))" and start playing with it. I'd suggest running " code{"cargo build"}" right away because the first invocation will download, build and cache all the dependencies from sources and it might take considerable time. Anyway, if you'll go straight for " code{"cargo run"}" it will build everything automatically as well. Also, if you're going for active development I'd highly recommend "(link("cargo-watch", "crates.io/crates/cargo-watch"))" to automatically rebuild and rerun the project on changes with a simple " code{"cargo watch -x run"}}
                p{"Beware that the default config intentionally omits the service worker to speed up the local host build for development purposes, but you can include it by adding " code{r#"--features="sw""#}" flag. Another available feature is " code{"containerize"}" which builds host into a wasi container instead of the local executable. You can enable both with " code{r#"--features="sw containerize""#}". Also, you can add the release flag " code{"--release"}" to get faster and smaller binaries, just beware that compilation will take longer. Last but not least - " code{r#"cargo run --features="webview""#}" will start the host and connect a webview to it."}
                p{"That's it, you're all set to build speedy modern apps! This stack allows mixing Rust and TypeScript on pretty much any platform in any way you want without compromising types or memory safety. Some ideas how you can extend the architecture:"}
                ul{
                    li{"embed TS/JS runtimes like "(link("QuickJS", "bellard.org/quickjs/quickjs"))" or "(link("Deno", "deno.com/blog/roll-your-own-javascript-runtime"))" into the host"}
                    li{"compile Rust and/or TypeScript code for cloud JS platforms like cloudflare workers while testing locally with "(link("workerd", "github.com/cloudflare/workerd"))}
                    li{"embed a "(link("GlueSQL db", "github.com/gluesql/gluesql"))" into the service worker and/or the host, or just "(link("connect the wasi host to an external DB", "github.com/WasmEdge/wasmedge-db-examples"))} 
                    li{"supercharge the UI with "(link("WebGPU", "wgpu.rs"))" for complex graphics and even "(link("games", "bevyengine.org"))}
                    li{"run AI stuff anywhere with "(link("TensorFlow.js", "tensorflow.org/js"))" or "(link("wasi-nn", "github.com/WebAssembly/wasi-nn"))}
                }
                p{"And keep in mind that rust can run on pretty much any platform/device efficiently so only your imagination is the limit. I think this project showcases cross-platform development with rust well enough to embrace it's " b{"powers"}" and maybe even start building something with it, but if not - please send your doubts to " code{"edezhic@gmail.com"}", I'm curious what it might be missing to win your heart!"}
            } 
        )
    }
    
}