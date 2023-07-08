fn main() {
    // by default cargo only tracks changes in rust sources but we want to track TS/JS and other stuff as well
    tools::track_non_rust_changes(&["ui", "sw", "host"]); // nah???

    tools::bundle_and_transpile_ts("ui", cfg!(feature = "sw"));
    tools::bundle_and_compile_scss("ui");

    if cfg!(feature = "sw") {
        // compile service worker rust code into wasm and bundle main.ts with it
        tools::build_wasm_with_bindings_and_combine_with_ts("sw");
    }
    
    // build host for the local platform for development/debugging
    tools::build("host");
}
