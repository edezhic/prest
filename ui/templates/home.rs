use crate::external_link::render as link;
pub fn render() -> maud::Markup {
    maud::html!(
        h1{b{"p"}"rogressive " b{"w"}"eb " b{"r"}"u" b{"s"}"t " b{"app"}"lication"}
        ."highlight summary"{
            p{b{"Progressive"}" app that starts as a web page and can bootstrap up to a native one"}
            p{"based on " b{"Web"}" standards such as HTML, CSS, TS/JS, APIs, WASM, WASI et cetera"}
            p{"written in " b{"Rust"}" for safety and speed on a wide range of platforms"}
        }
        //(forward_button("more about PWAs and Rust", "motivations"))

        ."columns-row" {
            ."column" {
                img."svg fill" src="/browser.svg";
                h2{"browser"}
                small{b{"WebAssembly"}" compilation and generated "b{"TypeScript"}" bindings provide flexibility in the choice of the front-end tooling while preserving memory and type safety"}
            }
            ."column" {
                img."svg stroke" src="/cloud.svg";
                h2{"cloud"}
                small{b{"WASI"}" target enables extremely small and performant containers which can even embed JS runtimes"}
            }
            ."column" {
                img."svg fill" src="/mobile.svg";
                h2{"native"}
                small{b{"linux"}", "b{"windows"}", "b{"mac"}", "b{"android"}", "b{"ios"}" and other platforms can use the same UI based on native browsers and run service worker or use embedded/remote host"}
            }
        }
        //(forward_button("more about platform support", "platforms"}

        ."highlight architecture"{"Cross-platform frameworks usually require heavy runtimes to run the same code everywhere, but not with rust! Currently pwrsapp includes:"}
        
        ."columns-row" {
            ."column" {
                img."svg stroke" src="/ui_shapes.svg";
                h2{"UI"}
                small{"interface rendering and interactions"}
            }
            ."column" {
                img."svg fill" src="/cogs.svg";
                h2{"service worker"}
                small{"runs on the client side to enable offline work"}
            }
            ."column" {
                img."svg fill" src="/host.svg";
                h2{"host"}
                small{"runs natively in the cloud, desktop and mobile"}
            }
        }
        ."highlight" {
            p{"the whole app packed in a WASM+WASI binary is just " b{"~2.5mb"}"! This is pretty nuts considering that it includes the server, icons, service worker, and even duplicated UI because both host and sw include it while host also includes the full sw bundle."}
        }
        //(forward_button("more about pwrsapp internals", "internals"}

        h2{"Getting started"}
        ."highlight details"{
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
