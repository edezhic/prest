pub fn render() -> maud::Markup {
    maud::html!(
        html {
            head {
                title {"PWRS app"}
                link rel="icon" href="/favicon.ico" {}
                link rel="manifest" href="/.webmanifest" {}
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/water.css@2/out/dark.css" {}
                @if cfg!(any(target_arch = "wasm32", feature = "sw")) { script src="/include_sw.js" {} }
                meta name="viewport" content="width=device-width, initial-scale=1.0";
            }
            body {
                h1{b{"P"}"rogressive " b{"W"}"eb " b{"R"}"u" b{"S"}"t " b{"app"}"lication"}
                ."highlight"{
                    p{b{"Progressive"}" app that starts as a web page and can bootstrap up to a native one"}
                    p{"based on " b{"Web"}" standards such as HTML, CSS, TS/JS, Web APIs, WASM et cetera"}
                    p{"written in " b{"Rust"}" for safety and speed on a wide range of platforms"}
                }
            }
        }
    )
}   
