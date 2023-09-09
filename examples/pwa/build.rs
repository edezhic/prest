fn main() {
    #[cfg(feature = "host")]
    prest::build::generate_pwa_assets();
}
