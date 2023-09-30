mod sw;
mod wasm_bindgen;
mod webmanifest;
#[cfg(feature = "typescript")]
mod swc;

use std::{
    format as f,
    fs::{create_dir_all, read_to_string, rename, write},
    process::Command,
    time::Instant,
};

pub static SW_TARGET_DIR: &str = "target_sw";

#[cfg(debug_assertions)]
pub static PROFILE: &str = "debug";
#[cfg(not(debug_assertions))]
pub static PROFILE: &str = "release";

pub static WASM_UNK: &str = "wasm32-unknown-unknown";
pub static WASM_WASI: &str = "wasm32-wasi";

pub static DEFAULT_LOGO: &[u8] = include_bytes!("assets/logo.png");
pub static DEFAULT_FAVICON: &[u8] = include_bytes!("assets/favicon.ico");

#[cfg(feature = "typescript")]
pub fn bundle_ts(input: &str, filename: &str) {
    let start = Instant::now();
    track_non_rust_path(input); // track imports?
    let contents = swc::run(input, false, false).unwrap();
    write(out_path(filename), contents).unwrap();
    bench(&f!("{input} transpiled and bundled as {filename}"), start);
}

pub fn bundle_scss(input: &str, filename: &str) {
    let start = Instant::now();
    track_non_rust_path(input); // track imports?
    let contents = grass::from_path(input, &Default::default()).unwrap();
    write(out_path(filename), contents).unwrap();
    bench(&f!("{input} transpiled and bundled as {filename}"), start);
}

pub fn generate_pwa_assets() {
    use sw::append_sw_listeners;
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

    wasm_bindgen::run(SW_TARGET_DIR, lib_name, false).unwrap();
    // moving the built wasm binary
    rename(&f!("{target_path}_bg.wasm"), out_path("sw.wasm")).unwrap();
    // appending and saving js bindings
    let js = read_to_string(&f!("{target_path}.js")).unwrap();
    let js = append_sw_listeners(js);
    write(out_path("sw.js"), &js).unwrap();

    let manifest = webmanifest::compose(webmanifest::ManifestOptions::default());
    write(out_path(".webmanifest"), manifest).unwrap();
    write(out_path("logo.png"), DEFAULT_LOGO).unwrap();
    write(out_path("favicon.ico"), DEFAULT_FAVICON).unwrap();

    bench(&f!("built service worker"), start);
}

use once_cell::sync::Lazy;
static ASSETS_DIR: Lazy<String> = Lazy::new(|| {
    let dir = f!("{}/assets", std::env::var("OUT_DIR").unwrap());
    create_dir_all(&dir).unwrap();
    dir
});

fn out_path(filename: &str) -> String {
    f!("{}/{filename}", *ASSETS_DIR)
}

fn read_lib_name() -> String {
    use toml::{Table, Value};
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = &f!("{manifest_dir}/Cargo.toml");
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

// cargo:rerun-if-changed invalidates build caches if anything changes there
// so that it keeps track of TS and other sources as well as rust dependencies
fn track_non_rust_path(path: &str) {
    println!("cargo:rerun-if-changed={path}");
}

fn bench(message: &str, start: Instant) {
    println!(
        "cargo:warning={}",
        format!("{} in {}ms", message, start.elapsed().as_millis())
    );
}

#[allow(dead_code)]
pub fn debug(message: &str) {
    println!("cargo:warning=(debug): {message}");
}
