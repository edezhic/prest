fn main() {
    // by default cargo tracks changes only in rust sources, but we want to track TS/JS and other stuff as well
    prest::build::track_non_rust_changes(&["main.ts", "ui"]);

    // trigger build of the assets only when building the host
    if prest::build::detect_wasm_build() { return }
    
    prest::build::bundle_ts("ui/main.ts", "ui");
    prest::build::bundle_scss("ui/main.scss", "ui");
    
    if cfg!(feature = "sw") {
        prest::build::append_sw_registration("ui.js");
        // compile service worker rust code into wasm and bundle main.ts with it
        prest::build::bundle_sw("shared", "sw.ts");
    }
    
}
