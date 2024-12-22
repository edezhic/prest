use prest_build::*;
fn main() {
    default_cfg_aliases();
    build_pwa(PWAOptions::new()).unwrap();
}
