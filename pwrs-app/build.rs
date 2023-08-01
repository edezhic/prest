fn main() {
    cfg_aliases::cfg_aliases! {
        host: { not(target_arch = "wasm32") },
        sw: { target_arch = "wasm32" },
    }
    
    pwrs_build::track_non_rust_changes(&["styles.scss", "sw.ts"]);

    if pwrs_build::detect_sw_build() { return }
    
    pwrs_build::bundle_scss("styles.scss", "styles");
    
    if cfg!(feature = "sw") {
        pwrs_build::append_sw_registration("pub/include_sw.js");
        pwrs_build::bundle_sw();
    }
    
}
