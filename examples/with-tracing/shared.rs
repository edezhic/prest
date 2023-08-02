use pwrs::*;
#[allow(dead_code)]
pub fn service() -> Router {
    Router::new().route("/", render!(homepage))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub async fn serve(host: &str, event: pwrs_sw::FetchEvent) {
    pwrs_sw::process_fetch_event(service, host, event).await
}

mod homepage {
    pub fn render() -> pwrs::Markup {
        maud::html!(
            html {
                head {
                    title {"PWRS with tracing"}
                    link rel="icon" href="/favicon.ico" {}
                    link rel="manifest" href="/.webmanifest" {}
                    link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/water.css@2/out/dark.css" {}
                    @if cfg!(any(target_arch = "wasm32", feature = "sw")) { script src="/include_sw.js" {} }
                    meta name="viewport" content="width=device-width, initial-scale=1.0";
                }
                body {
                    h1{"PWRS"}
                    h2{"powers for you app!"}
                    p{b{"Progressive"}" application that starts as a webpage and can bootstrap up to native"}
                    p{"based on " b{"Web"}" standards such as HTML, CSS, TS/JS, Web APIs, WASM et cetera"}
                    p{"written in " b{"RuSt"}" for safety and speed on a wide range of platforms"}
                    
                }
            }
        )
    }       
}
