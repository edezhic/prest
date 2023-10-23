use super::{bench, out_path, read_lib_name, PROFILE, SW_TARGET_DIR, WASM_UNK};
use std::{
    format as f,
    fs::{read_to_string, rename, write},
    process::Command,
    time::Instant,
};
use wasm_bindgen_cli_support::Bindgen;
use webmanifest::{DisplayMode, Icon, Manifest};

static DEFAULT_LOGO: &[u8] = include_bytes!("dist/logo.png");
static DEFAULT_FAVICON: &[u8] = include_bytes!("dist/favicon.ico");
static LISTENER_TEMPLATE: &str = "self.addEventListener('NAME', event => LISTENER);\n";

pub struct PWAOptions<'a> {
    pub build_features: &'a str,
    pub types_path: Option<&'a str>,
    pub listeners: Vec<(&'a str, &'a str)>,
    pub manifest: ManifestOptions<'a>,
}
impl Default for PWAOptions<'_> {
    fn default() -> Self {
        Self {
            build_features: "sw",
            types_path: None,
            listeners: vec![
                ("install", "event.waitUntil(Promise.all([__wbg_init('/dist/sw.wasm'), self.skipWaiting()]))"),
                ("activate", "event.waitUntil(self.clients.claim())"),
                ("fetch", "handle_fetch(self, event)")
            ],
            manifest: ManifestOptions::default(),
        }
    }
}

pub struct ManifestOptions<'a> {
    pub name: String,
    pub desc: String,
    pub background: String,
    pub theme: String,
    pub start: String,
    pub display: DisplayMode,
    pub icons: Vec<Icon<'a>>,
}
impl Default for ManifestOptions<'_> {
    fn default() -> Self {
        Self {
            name: std::env::var("CARGO_PKG_NAME").unwrap(),
            desc: if let Ok(desc) = std::env::var("CARGO_PKG_DESCRIPTION") {
                desc
            } else {
                "An installable web application".to_owned()
            },
            background: "#1e293b".to_owned(),
            theme: "#a21caf".to_owned(),
            start: "/".to_owned(),
            display: DisplayMode::Standalone,
            icons: vec![Icon::new("logo.png", "512x512")],
        }
    }
}

pub fn build_pwa(opts: PWAOptions) {
    let start = Instant::now();
    let lib_name = &read_lib_name();
    let target_dir = &f!("{SW_TARGET_DIR}/{WASM_UNK}/{PROFILE}");
    let target_path = &f!("{target_dir}/{lib_name}");

    // build in a separate target dir to avoid build deadlock with the host
    let mut cmd = Command::new("cargo");
    cmd.arg("rustc")
        .arg("--lib")
        .args(["--crate-type", "cdylib"])
        .arg("--no-default-features")
        .args(["--features", &opts.build_features])
        .args(["--target", WASM_UNK])
        .args(["--target-dir", SW_TARGET_DIR]);
    if !cfg!(debug_assertions) {
        cmd.arg("--release");
    }
    assert!(cmd.status().expect("finished SW wasm build").success());

    // generate bindings for the wasm binary
    Bindgen::new()
        .input_path(f!("{target_path}.wasm"))
        .web(true)
        .unwrap()
        .typescript(opts.types_path.is_some())
        .remove_name_section(true)
        .remove_producers_section(true)
        .omit_default_module_path(true)
        .generate(target_dir)
        .unwrap();

    // move the processed wasm binary into final dist
    rename(&f!("{target_path}_bg.wasm"), out_path("sw.wasm")).unwrap();

    // append event listeners and save js bindings
    let mut js = read_to_string(&f!("{target_path}.js")).unwrap();
    for listener in opts.listeners {
        js += LISTENER_TEMPLATE
            .replace("NAME", listener.0)
            .replace("LISTENER", listener.1)
            .as_str();
    }
    write(out_path("sw.js"), &js).unwrap();

    // compose .webmanifest with app metadata
    write(out_path(".webmanifest"), gen_manifest(opts.manifest)).unwrap();

    // at least one logo is required for PWA installability
    write(out_path("logo.png"), DEFAULT_LOGO).unwrap();

    // for completeness?
    write(out_path("favicon.ico"), DEFAULT_FAVICON).unwrap();

    bench(&f!("built PWA dist"), start);
}

fn gen_manifest(opts: ManifestOptions) -> String {
    let mut manifest = Manifest::builder(&opts.name)
        .description(&opts.desc)
        .bg_color(&opts.background)
        .theme_color(&opts.theme)
        .start_url(&opts.theme)
        .display_mode(opts.display.clone());
    for icon in &opts.icons {
        manifest = manifest.icon(icon);
    }
    manifest.build().unwrap()
}
