fn main() {
    if cfg!(all(not(target_arch = "wasm32"), feature = "sw")) {
        pwrs_build::append_sw_registration("pub/include_sw.js");
        pwrs_build::bundle_sw();
    }
}
