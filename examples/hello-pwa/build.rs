fn main() {
    #[cfg(feature = "host")]
    prest::build_pwa(Default::default());
}
