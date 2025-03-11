fn main() {
    prest_build::default_cfg_aliases();
    prest_build::bundle_ts();
    prest_build::bundle_sass("ui/preset.css").expect("default styles should build");
    prest_build::bundle_sass("ui/admin.scss").expect("admin styles should build");
}
