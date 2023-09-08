fn main() {
    #[cfg(feature = "host")]
    pwrs::build::generate_pwa_assets();
}
