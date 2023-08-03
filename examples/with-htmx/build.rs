fn main() {
    // by default cargo tracks changes only in rust sources, but we want to track TS/JS and other stuff as well
    pwrs::build::track_non_rust_changes(&["main.ts", "ui"]);

    // trigger build of the assets only when building the host
    if pwrs::build::detect_sw_build() { return }
    
    pwrs::build::bundle_ts("ui/main.ts", "ui");
    pwrs::build::bundle_scss("ui/main.scss", "ui");
    
    if cfg!(feature = "sw") {
        pwrs::build::append_sw_registration("ui.js");
        // compile service worker rust code into wasm and bundle main.ts with it
        pwrs::build::bundle_sw("shared", "sw.ts");
    }
    
}
