fn main() {
    #[cfg(feature = "host")] {
        std::fs::copy("./styles.css", prest::out_path("styles.css")).unwrap();
        #[cfg(not(debug_assertions))]
        prest::build_pwa(Default::default());
    }
}
