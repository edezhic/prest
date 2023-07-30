fn main() {
    cfg_aliases::cfg_aliases! {
        host: { not(target_arch = "wasm32") },
        sw: { target_arch = "wasm32" },
    }
    
    // trigger build of the assets only when building the host
    if std::env::var("CARGO_CFG_TARGET_ARCH").unwrap() == "wasm32" { return }
    
    // by default cargo tracks changes only in rust sources, but we want to track TS/JS and other stuff as well
    pwrs_build::track_non_rust_changes(&["main.ts", "ui"]);

    pwrs_build::bundle_and_transpile_ui(cfg!(feature = "sw"));
    
    if cfg!(feature = "sw") {
        // compile service worker rust code into wasm and bundle main.ts with it
        pwrs_build::build_sw();
    }
    
}
