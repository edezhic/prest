fn main() {
    if cfg!(all(not(target_arch = "wasm32"), feature = "sw")) {
        pwrs::build::append_sw_registration("include_sw.js");
        pwrs::build::bundle_sw("shared", "sw.ts");
    }
}
