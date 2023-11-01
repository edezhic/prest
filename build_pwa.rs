use crate::out_path;
use std::{
    format as f,
    fs::{read_to_string, rename, write},
    process::Command,
    time::Instant,
};
use wasm_bindgen_cli_support::Bindgen;
use webmanifest::{DisplayMode, Icon, Manifest};

#[cfg(debug_assertions)]
const PROFILE: &str = "debug";
#[cfg(not(debug_assertions))]
const PROFILE: &str = "release";

// Replace with dynamic target-based path? An example how to figure out target dir:
// https://github.com/dtolnay/cxx/blob/306019c5a7434aa7424a83720a09c40e1ea12343/gen/build/src/target.rs
static SW_TARGET_DIR: &str = "target_sw";

static DEFAULT_LOGO: &[u8] = include_bytes!("assets/logo.png");
static DEFAULT_FAVICON: &[u8] = include_bytes!("assets/favicon.ico");
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
                ("install", "event.waitUntil(Promise.all([__wbg_init('/sw.wasm'), self.skipWaiting()]))"),
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
    let target_dir = &f!("{SW_TARGET_DIR}/wasm32-unknown-unknown/{PROFILE}");
    let target_path = &f!("{target_dir}/{lib_name}");

    // build in a separate target dir to avoid build deadlock with the host
    let mut cmd = Command::new("cargo");
    cmd.arg("rustc")
        .arg("--lib")
        .args(["--crate-type", "cdylib"])
        .arg("--no-default-features")
        .args(["--features", &opts.build_features])
        .args(["--target", "wasm32-unknown-unknown"])
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

    println!(
        "cargo:warning={}",
        f!("composed PWA in {}ms", start.elapsed().as_millis())
    );
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

fn read_lib_name() -> String {
    use toml::{Table, Value};
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = &format!("{manifest_dir}/Cargo.toml");
    let manifest = read_to_string(manifest_path).unwrap();
    let parsed = manifest.parse::<Table>().unwrap();
    let lib_name = if let Value::Table(lib_table) = &parsed["lib"] {
        if lib_table.contains_key("name") {
            lib_table["name"].as_str().unwrap().to_owned()
        } else {
            parsed["package"]["name"].as_str().unwrap().to_owned()
        }
    } else {
        parsed["package"]["name"].as_str().unwrap().to_owned()
    };
    lib_name.replace("-", "_")
}
