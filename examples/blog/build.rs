fn main() {
    #[cfg(all(feature = "host", not(debug_assertions)))] {
        prest::build_pwa(Default::default());
        prest::distribute("./styles.css");
    }
}
