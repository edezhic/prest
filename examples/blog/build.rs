fn main() {
    #[cfg(all(feature = "host", not(debug_assertions)))] 
    prest::build::generate_pwa_assets();

    #[cfg(feature = "host")] { 
        prest::build::bundle_ts("./script.ts", "ui.js");
        prest::build::bundle_scss("./styles.scss", "ui.css");
    }
}
