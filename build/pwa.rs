// DUPLICATED FROM ../lib.rs DUE TO AN ISSUE WITH RUST-ANALYZER FAILING TO RESOLVE #[path...] ATTRIBUTE
pub fn is_pwa() -> bool {
    #[cfg(target_arch = "wasm32")]
    return true;
    #[cfg(not(target_arch = "wasm32"))]
    {
        #[cfg(debug_assertions)]
        return std::env::var("PWA").map_or(false, |v| v == "debug");
        #[cfg(not(debug_assertions))]
        return std::env::var("PWA").map_or(true, |v| v == "release");
    }
}