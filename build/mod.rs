mod sw;
mod swc;
mod wasm_bindgen;
mod webmanifest;
use std::{format as f, fs};
use std::io::Write;
use std::process::Command;
use std::{
    fs::{read_to_string, remove_file, rename, write, OpenOptions},
    time::Instant,
};

use crate::REGISTER_SW_SNIPPET;
use sw::append_sw_listeners;

#[cfg(debug_assertions)]
pub static PROFILE: &str = "debug";
#[cfg(not(debug_assertions))]
pub static PROFILE: &str = "release";

pub static WASM_UNK: &str = "wasm32-unknown-unknown";
pub static WASM_WASI: &str = "wasm32-wasi";

pub static DEFAULT_LOGO: &[u8] = include_bytes!("assets/logo.png");
pub static DEFAULT_FAVICON: &[u8] = include_bytes!("assets/favicon.ico");

// cargo:rerun-if-changed invalidates build caches if anything changes there
// so that it keeps track of TS and other sources as well as rust dependencies
pub fn track_non_rust_changes(paths: &[&str]) {
    for path in paths.iter() {
        println!("cargo:rerun-if-changed={path}");
    }
}

pub fn detect_wasm_build() -> bool {
    std::env::var("CARGO_CFG_TARGET_ARCH").unwrap() == "wasm32"
}

pub fn append_sw_registration(includer_name: &str) {
    let includer_path = &f!("pub/{includer_name}");
    let mut js_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(includer_path)
        .unwrap();

    if read_to_string(includer_path).unwrap().contains(&REGISTER_SW_SNIPPET) {
        return;
    }

    js_file.write(REGISTER_SW_SNIPPET.as_bytes()).unwrap();
}

pub fn bundle_ts(input: &str, output_name: &str) {
    let start = Instant::now();
    let js = swc::run(input, false, false).unwrap();
    let output = &f!("pub/{output_name}.js");
    write(output, js).unwrap();
    bench(&f!("{input} transpiled and bundled into {output}"), start);
}

pub fn bundle_scss(input: &str, output_name: &str) {
    let start = Instant::now();
    let css = grass::from_path(input, &Default::default()).unwrap();
    let output = &f!("pub/{output_name}.css");
    write(output, css).unwrap();
    bench(&f!("{input} transpiled and bundled into {output}"), start);
}

pub fn generate_pwa_assets() {
    let start = Instant::now();
    let lib_name = "serve"; // TODO: READ THE NAME FROM CARGO.TOML?
    let out_dir = &std::env::var("OUT_DIR").unwrap();
    let out_path = &f!("{out_dir}/assets");
    fs::create_dir_all(out_path).unwrap();
    debug(&f!("Assets will be saved at {out_path}"));
    // changing default target dir to avoid deadlock with other workspace builds including the host
    let target_dir = "target_sw";
    let target_path = &f!("{target_dir}/{WASM_UNK}/{PROFILE}/{lib_name}");
    let mut cmd = Command::new("cargo");
    cmd.arg("rustc")
        .arg("--lib")
        .args(["--crate-type", "cdylib"])
        .arg("--no-default-features")
        .args(["--features", "sw"])
        .args(["--target", WASM_UNK])
        .args(["--target-dir", target_dir]);
    if !cfg!(debug_assertions) {
        cmd.arg("--release");
    }
    assert!(cmd.status().expect("finished wasm build").success());

    wasm_bindgen::run(target_dir, lib_name, false).unwrap();
    // moving the built wasm binary
    rename(&f!("{target_path}_bg.wasm"), &f!("{out_path}/sw.wasm")).unwrap();
    // appending and saving js bindings
    let js = read_to_string(&f!("{target_path}.js")).unwrap();
    let js = append_sw_listeners(js);
    write(f!("{out_path}/sw.js"), &js).unwrap();

    let manifest = webmanifest::compose();
    write(&f!("{out_path}/.webmanifest"), manifest).unwrap();

    write(&f!("{out_path}/logo.png"), DEFAULT_LOGO).unwrap();
    write(&f!("{out_path}/favicon.ico"), DEFAULT_FAVICON).unwrap();

    bench(&f!("built service worker"), start);
}

pub fn bundle_sw(lib_name: &str, sw_path: &str) {
    let start = Instant::now();
    // changing default target dir to avoid deadlock with other workspace builds
    let target_dir = "target/sw";
    let target_path = &f!("{target_dir}/{WASM_UNK}/{PROFILE}/{lib_name}");
    let mut cmd = Command::new("cargo");
    cmd.arg("rustc")
        .arg("--lib")
        .args(["--crate-type", "cdylib"])
        .args(["--target", WASM_UNK])
        .args(["--target-dir", target_dir]);
    if !cfg!(debug_assertions) {
        cmd.arg("--release");
    }
    assert!(cmd.status().expect("finished wasm build").success());

    wasm_bindgen::run(target_dir, lib_name, true).unwrap();

    rename(&f!("{target_path}_bg.wasm"), &f!("pub/sw.wasm")).unwrap();
    // moving lib types into the crate for development convenience
    rename(&f!("{target_path}.d.ts"), &f!("./{lib_name}.d.ts")).unwrap();

    // moving bindings into the crate to include in the bundle
    rename(&f!("{target_path}.js"), &f!("./{lib_name}.js")).unwrap();
    // bundle and transpile main.ts
    let js = swc::run(sw_path, PROFILE == "release", true).unwrap();
    write(f!("pub/sw.js"), &js).unwrap();

    // clean up generated lib.js bindings
    remove_file(&f!("./{lib_name}.js")).unwrap();

    bench(
        &f!("service worker compiled into wasm and bundled with typescript"),
        start,
    );
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
