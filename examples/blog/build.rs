fn main() {
    #[cfg(not(debug_assertions))]
    prest::build_pwa(prest::PWAOptions::default());
}
