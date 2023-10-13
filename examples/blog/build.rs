fn main() {
    #[cfg(all(feature = "host", not(debug_assertions)))]
    prest::build::generate_pwa_assets();

    #[cfg(feature = "host")] 
    prest::build::include_asset("./styles.css");
}
