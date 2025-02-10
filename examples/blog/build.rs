use prest_build::*;
fn main() {
    default_cfg_aliases();
    bundle_ts();
    build_pwa(PWAOptions::new()).unwrap();
}
