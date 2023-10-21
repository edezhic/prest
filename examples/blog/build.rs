fn main() {
    #[cfg(feature = "host")] {
        prest::generate_pwa_assets();
        prest::distribute("./styles.css");
    }
}
