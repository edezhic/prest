fn main() {
    #[cfg(feature = "host")]
    prest::build::generate_pwa_assets();

    #[cfg(feature = "host")] 
     prest::build::bundle_scss("./styles.scss", "ui.css");
}
