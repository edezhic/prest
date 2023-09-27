fn main() {
    #[cfg(feature = "host")] {
        prest::build::generate_pwa_assets();
        prest::build::bundle_ts("ui/main.ts", "ui.js");
        prest::build::bundle_scss("ui/main.scss", "ui.css");
    }
}
