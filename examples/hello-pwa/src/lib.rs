use prest::*;

pub fn shared() -> Router {
    let content = html!(
        (Head::example("Hello PWA"))
        body {h1{"Hello from PWA!"} (Scripts::default())}
    );
    route("/",get(content))
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main() {
    shared().serve()
}
