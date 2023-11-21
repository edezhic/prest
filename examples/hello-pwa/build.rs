fn main() {
    #[cfg(feature = "default")]
    prest::build_pwa(prest::PWAOptions::default());
}
