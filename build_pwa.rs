use crate::{find_target_dir, out_path, RELEASE};
use std::{
    env, format as f,
    fs::{read_to_string, rename, write},
    process::Command,
    time::Instant,
};
use webmanifest::{DisplayMode, Icon, Manifest};

pub struct PWAOptions<'a> {
    pub release_only: bool,
    pub listeners: Vec<(&'a str, &'a str)>,
    pub name: String,
    pub desc: String,
    pub background: String,
    pub theme: String,
    pub start: String,
    pub display: DisplayMode,
    pub icons: Vec<Icon<'a>>,
}
impl Default for PWAOptions<'_> {
    fn default() -> Self {
        Self {
            release_only: true,
            listeners: vec![
                (
                    "install",
                    "event.waitUntil(Promise.all([__wbg_init('/sw.wasm'), self.skipWaiting()]))",
                ),
                ("activate", "event.waitUntil(self.clients.claim())"),
                ("fetch", "handle_fetch(self, event)"),
            ],
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

const SW_TARGET: &str = "service-worker";

static LOGO: &[u8] = include_bytes!("assets/logo.png");

static LISTENER_TEMPLATE: &str = "self.addEventListener('NAME', event => LISTENER);\n";

pub fn build_pwa(opts: PWAOptions) {
    if env::var("SELF_PWA_BUILD").is_ok() || (opts.release_only && !RELEASE) {
        return;
    }
    let start = Instant::now();
    let lib_name = &read_lib_name();
    let target_dir = sw_target_dir();
    let profile_dir = match RELEASE {
        true => "release",
        false => "debug",
    };
    let profile_path = &f!("{target_dir}/wasm32-unknown-unknown/{profile_dir}");
    let lib_path = &f!("{profile_path}/{lib_name}");

    // build in a separate target dir to avoid build deadlock with the host
    let mut cmd = Command::new("cargo");
    cmd.env("SELF_PWA_BUILD", "true")
        .arg("rustc")
        .arg("--lib")
        .args(["--crate-type", "cdylib"])
        .args(["--target", "wasm32-unknown-unknown"])
        .args(["--target-dir", &target_dir]);
    //.arg("--")
    //.args(["--cfg", r#"'self_sw_build=true'"#]);
    if !cfg!(debug_assertions) {
        cmd.arg("--release");
    }
    assert!(cmd.status().expect("finished SW wasm build").success());

    // generate bindings for the wasm binary
    wasm_bindgen_cli_support::Bindgen::new()
        .input_path(f!("{lib_path}.wasm"))
        .web(true)
        .unwrap()
        .remove_name_section(true)
        .remove_producers_section(true)
        .omit_default_module_path(true)
        .generate(profile_path)
        .unwrap();

    // move the processed wasm binary into final dist
    rename(&f!("{lib_path}_bg.wasm"), out_path("sw.wasm")).unwrap();

    // append event listeners and save js bindings
    let mut js = read_to_string(&f!("{lib_path}.js")).unwrap();
    for listener in opts.listeners.iter() {
        js += LISTENER_TEMPLATE
            .replace("NAME", listener.0)
            .replace("LISTENER", listener.1)
            .as_str();
    }
    write(out_path("sw.js"), &js).unwrap();

    // compose .webmanifest with app metadata
    write(out_path(".webmanifest"), gen_manifest(opts)).unwrap();

    // at least one logo is required for PWA installability
    write(out_path("logo.png"), LOGO).unwrap();

    println!(
        "cargo:warning={}",
        f!("composed PWA in {}ms", start.elapsed().as_millis())
    );
}

fn gen_manifest(opts: PWAOptions) -> String {
    let mut manifest = webmanifest::Manifest::builder(&opts.name)
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

fn sw_target_dir() -> String {
    if let Some(dir) = find_target_dir() {
        dir + "/" + SW_TARGET
    } else {
        "target/".to_owned() + SW_TARGET
    }
}
