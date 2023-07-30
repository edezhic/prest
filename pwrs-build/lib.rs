mod swc;
mod wasm_bindgen;
use std::format as f;
use std::process::Command;
use std::{
    fs::{remove_file, rename, write},
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

pub fn bundle_and_transpile_ui(register_sw: bool) {
    let start = Instant::now();
    let mut js = swc::run("ui/main.ts", false, false).unwrap();
    if register_sw {
        js = SW_SNIPPET.to_owned() + js.as_str();
    }
    write(f!("pub/ui.js"), &js).unwrap();

    let css = grass::from_path("ui/main.scss", &Default::default()).unwrap();
    write(f!("pub/ui.css"), css).unwrap();
    
    bench(&f!("UI transpiled and bundled"), start);
}

pub fn build_sw() {
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
    let js = swc::run("./main.ts", PROFILE == "release", true).unwrap();
    write(f!("pub/sw.js"), &js).unwrap();

    // clean up lib.js bindings
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
