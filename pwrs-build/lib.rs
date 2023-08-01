mod swc;
mod wasm_bindgen;
use std::format as f;
use std::io::Write;
use std::process::Command;
use std::{
    fs::{remove_file, rename, write, read_to_string, OpenOptions},
    time::Instant,
};

#[cfg(debug_assertions)]
pub static PROFILE: &str = "debug";
#[cfg(not(debug_assertions))]
pub static PROFILE: &str = "release";

pub static WASM_UNK: &str = "wasm32-unknown-unknown";
pub static WASM_WASI: &str = "wasm32-wasi";

static SW_SNIPPET: &str = "if ('serviceWorker' in navigator) navigator.serviceWorker.register('sw.js', {type: 'module'}); \n";

// cargo:rerun-if-changed invalidates build caches if anything changes there
// so that it keeps track of TS and other sources as well as rust dependencies
pub fn track_non_rust_changes(paths: &[&str]) {
    for path in paths.iter() {
        println!("cargo:rerun-if-changed={path}");
    }
}

pub fn detect_sw_build() -> bool {
    std::env::var("CARGO_CFG_TARGET_ARCH").unwrap() == "wasm32"
}

pub fn append_sw_registration(path: &str) {
    let mut js_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap();

    if read_to_string(path).unwrap().contains(SW_SNIPPET) { return }

    js_file
        .write(SW_SNIPPET.as_bytes())
        .unwrap();
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

pub fn bundle_sw() {
    let start = Instant::now();
    let mut cmd = Command::new("cargo");
    cmd.arg("rustc")
        .arg("--lib")
        .args(["--crate-type", "cdylib"])
        .args(["--target", WASM_UNK])
        // changing default target dir to avoid deadlock with other workspace builds
        .args(["--target-dir", &format!("target/sw")]);
    if !cfg!(debug_assertions) {
        cmd.arg("--release");
    }
    assert!(cmd.status().expect("finished wasm build").success());
    
    wasm_bindgen::run().unwrap();

    rename(
        &f!("target/sw/{WASM_UNK}/{PROFILE}/lib_bg.wasm"),
        &f!("pub/sw.wasm"),
    )
    .unwrap();
    // moving lib types into the crate for development convenience
    rename(
        &f!("target/sw/{WASM_UNK}/{PROFILE}/lib.d.ts"),
        &f!("./lib.d.ts"),
    )
    .unwrap();

    // moving bindings into the crate to include in the bundle
    rename(
        &f!("target/sw/{WASM_UNK}/{PROFILE}/lib.js"),
        &f!("./lib.js"),
    )
    .unwrap();
    // bundle and transpile main.ts
    let js = swc::run("./sw.ts", PROFILE == "release", true).unwrap();
    write(f!("pub/sw.js"), &js).unwrap();

    // clean up generated lib.js bindings
    remove_file(&f!("./lib.js")).unwrap();

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
