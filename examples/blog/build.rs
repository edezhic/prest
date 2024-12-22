use prest_build::*;
fn main() {
    default_cfg_aliases();
    bundle_ts("src/ui/prism.js").unwrap();
    bundle_sass("src/ui/prism-tomorrow.css").unwrap();
    build_pwa(PWAOptions::new()).unwrap();
}
