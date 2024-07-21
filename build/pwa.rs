use crate::*;
use anyhow::Result;
use std::{
    env, format as f,
    fs::{read_to_string, rename, write},
    process::Command,
    time::Instant,
};
use webmanifest::{DisplayMode, Icon, Manifest};

pub struct PWAOptions<'a> {
    pub listeners: Vec<(&'a str, &'a str)>,
    pub name: String,
    pub desc: String,
    pub background: String,
    pub theme: String,
    pub start: String,
    pub display: DisplayMode,
    pub icons: Vec<Icon<'a>>,
    pub debug_pwa: bool,
}
impl Default for PWAOptions<'_> {
    fn default() -> Self {
        Self {
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
            debug_pwa: false,
        }
    }
}

impl PWAOptions<'_> {
    pub fn debug_pwa(mut self) -> Self {
        self.debug_pwa = true;
        self
    }
}

const SW_TARGET: &str = "service-worker";

static LOGO: &[u8] = include_bytes!("default-logo.png");

static LISTENER_TEMPLATE: &str = "self.addEventListener('NAME', event => LISTENER);\n";

pub fn build_pwa(opts: PWAOptions) -> Result<()> {
    if env::var("SELF_PWA_BUILD").is_ok() || (!opts.debug_pwa && !is_pwa()) {
        return Ok(());
    }
    let start = Instant::now();
    let lib_name = &read_lib_name()?;
    let target_dir = sw_target_dir();
    let profile_dir = match cfg!(debug_assertions) {
        true => "debug",
        false => "release",
    };
    let profile_path = &f!("{target_dir}/wasm32-unknown-unknown/{profile_dir}");
    let lib_path = &f!("{profile_path}/{lib_name}");

    // build in a separate target dir to avoid build deadlock with the host
    let mut cmd = Command::new("cargo");
    cmd.env("SELF_PWA_BUILD", "true")
        .arg("rustc")
        .arg("--lib")
        .args(["--crate-type", "cdylib"])
        //.args(["--features", "traces html embed"])
        .args(["--target", "wasm32-unknown-unknown"])
        .args(["--target-dir", &target_dir]);

    if !cfg!(debug_assertions) {
        cmd.arg("--release");
    }

    assert!(cmd.status()?.success());

    // generate bindings for the wasm binary
    wasm_bindgen_cli_support::Bindgen::new()
        .input_path(f!("{lib_path}.wasm"))
        .web(true)?
        .remove_name_section(cfg!(not(debug_assertions)))
        .remove_producers_section(cfg!(not(debug_assertions)))
        .keep_debug(cfg!(debug_assertions))
        .omit_default_module_path(true)
        .generate(profile_path)?;

    // move the processed wasm binary into final dist
    rename(&f!("{lib_path}_bg.wasm"), out_path("sw.wasm"))?;

    // append event listeners and save js bindings
    let mut js = read_to_string(&f!("{lib_path}.js"))?;
    for listener in opts.listeners.iter() {
        js += LISTENER_TEMPLATE
            .replace("NAME", listener.0)
            .replace("LISTENER", listener.1)
            .as_str();
    }
    write(out_path("sw.js"), &js)?;

    // compose .webmanifest with app metadata
    write(out_path(".webmanifest"), gen_manifest(opts))?;

    // at least one logo is required for PWA installability
    write(out_path("logo.png"), LOGO)?;

    println!(
        "cargo:warning={}",
        f!("composed PWA in {}ms", start.elapsed().as_millis())
    );

    Ok(())
}

fn gen_manifest(opts: PWAOptions) -> String {
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

fn sw_target_dir() -> String {
    if let Some(dir) = find_target_dir() {
        dir + "/" + SW_TARGET
    } else {
        "target/".to_owned() + SW_TARGET
    }
}

// SHOULD BE SYNCED WITH THE SAME FN IN ../lib.rs
pub fn is_pwa() -> bool {
    #[cfg(target_arch = "wasm32")]
    return true;
    #[cfg(not(target_arch = "wasm32"))]
    {
        #[cfg(debug_assertions)]
        return std::env::var("PWA").map_or(false, |v| v == "debug");
        #[cfg(not(debug_assertions))]
        return std::env::var("PWA").map_or(true, |v| v == "release");
    }
}
