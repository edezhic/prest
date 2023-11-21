fn main() {
    #[cfg(all(feature = "default", not(debug_assertions)))]
    prest::build_pwa(prest::PWAOptions::default());
}
