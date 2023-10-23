fn main() {
    #[cfg(feature = "host")] {
        prest::build_pwa(Default::default());
        prest::distribute("./styles.css");
    }
}
