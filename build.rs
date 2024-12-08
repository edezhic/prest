fn main() {
    prest_build::default_cfg_aliases();
    prest_build::bundle_ts("ui/prest.js").expect("default bundle should build");
    prest_build::bundle_ts("ui/admin.tsx").expect("admin js bundle should build");
    prest_build::bundle_sass("ui/admin.scss").expect("admin styles should build");
}
