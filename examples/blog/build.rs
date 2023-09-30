fn main() {
    // DISABLED WHILE SETTING UP WASM TOOLCHAIN ON REPLIT
    // https://ask.replit.com/t/multiple-rust-toolchains-with-nix/69334 
    //#[cfg(all(feature = "host", not(debug_assertions)))] 
    //prest::build::generate_pwa_assets();

    #[cfg(feature = "host")] 
    prest::build::bundle_scss("./styles.scss", "ui.css");
}
