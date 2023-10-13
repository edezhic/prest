use super::{bench, out_path, read_lib_name, PROFILE, SW_TARGET_DIR, WASM_UNK};
use anyhow::Error;
use std::{
    format as f,
    fs::{read_to_string, rename, write},
    process::Command,
    time::Instant,
};

static DEFAULT_LOGO: &[u8] = include_bytes!("assets/logo.png");
static DEFAULT_FAVICON: &[u8] = include_bytes!("assets/favicon.ico");
static LISTENER_TEMPLATE: &str = "self.addEventListener('NAME', event => LISTENER);\n";
static DEFAULT_LISTENERS: [(&str, &str); 3] = [
    (
        "install",
        "event.waitUntil(Promise.all([__wbg_init('/sw.wasm'), self.skipWaiting()]))",
    ),
    ("activate", "event.waitUntil(self.clients.claim())"),
    ("fetch", "serve(self, event)"),
];

pub fn generate_pwa_assets() {
    let start = Instant::now();
    let lib_name = &read_lib_name();
    // changing default target dir to avoid deadlock with other workspace builds including the host
    let target_path = &f!("{SW_TARGET_DIR}/{WASM_UNK}/{PROFILE}/{lib_name}");
    let mut cmd = Command::new("cargo");
    cmd.arg("rustc")
        .arg("--lib")
        .args(["--crate-type", "cdylib"])
        .arg("--no-default-features")
        .args(["--features", "sw"])
        .args(["--target", WASM_UNK])
        .args(["--target-dir", SW_TARGET_DIR]);
    if !cfg!(debug_assertions) {
        cmd.arg("--release");
    }
    assert!(cmd.status().expect("finished wasm build").success());

    generate_bindings(SW_TARGET_DIR, lib_name, false).unwrap();
    // moving the built wasm binary
    rename(&f!("{target_path}_bg.wasm"), out_path("sw.wasm")).unwrap();
    // appending and saving js bindings
    let js = read_to_string(&f!("{target_path}.js")).unwrap();
    let js = append_sw_listeners(js);
    write(out_path("sw.js"), &js).unwrap();

    let manifest = build_webmanifest(ManifestOptions::default());
    write(out_path(".webmanifest"), manifest).unwrap();
    write(out_path("logo.png"), DEFAULT_LOGO).unwrap();
    write(out_path("favicon.ico"), DEFAULT_FAVICON).unwrap();

    bench(&f!("built service worker"), start);
}

use wasm_bindgen_cli_support::Bindgen;
fn generate_bindings(target_dir: &str, lib_name: &str, types: bool) -> Result<(), Error> {
    let mut bindgen = Bindgen::new();
    bindgen
        .input_path(f!("{target_dir}/{WASM_UNK}/{PROFILE}/{lib_name}.wasm"))
        .web(true)?
        .typescript(types)
        .remove_name_section(true)
        .remove_producers_section(true)
        .omit_default_module_path(true);
    bindgen.generate(f!("{target_dir}/{WASM_UNK}/{PROFILE}"))?;
    Ok(())
}

fn append_sw_listeners(mut bindings: String) -> String {
    for listener in DEFAULT_LISTENERS {
        bindings += LISTENER_TEMPLATE
            .replace("NAME", listener.0)
            .replace("LISTENER", listener.1)
            .as_str();
    }
    bindings
}

use webmanifest::{DisplayMode, Icon, Manifest};
struct ManifestOptions<'a> {
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
                "An installable web application".to_string()
            },
            background: "#1e293b".to_owned(),
            theme: "#a21caf".to_owned(),
            start: "/".to_owned(),
            display: DisplayMode::Standalone,
            icons: vec![Icon::new("logo.png", "512x512")],
        }
    }
}

fn build_webmanifest(opts: ManifestOptions) -> String {
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
