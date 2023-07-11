mod cargo;
mod swc;
mod wasm_bindgen;
use std::format as f;
use std::{
    fs::{remove_file, rename, write},
    path::PathBuf,
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

pub fn bundle_and_compile_scss(name: &str) {
    let start = Instant::now();
    let path = PathBuf::from(f!("{name}/main.scss"));
    let css = grass::from_path(path, &Default::default()).unwrap();
    write(f!("pub/{name}.css"), css).unwrap();
    bench(&f!("{name} styles compiled and bundled"), start);
}

pub fn bundle_and_transpile_ts(name: &str, register_sw: bool) {
    let start = Instant::now();
    let mut js = swc::run(name, false, false).unwrap();
    if register_sw {
        js = SW_SNIPPET.to_owned() + js.as_str();
        bench(&f!("{name} included sw snippet"), start);
    }
    write(f!("pub/{name}.js"), &js).unwrap();
    bench(&f!("{name} scripts transpiled and bundled"), start);
}

pub fn build_wasm_with_bindings_and_combine_with_ts(name: &str) {
    let start = Instant::now();
    cargo::build(name, Some(WASM_UNK));
    wasm_bindgen::run(name).unwrap();
    // no more processing for the wasm binary, just moving and renaming for convinience
    rename(
        &f!("target/{name}/{WASM_UNK}/{PROFILE}/{name}_bg.wasm"),
        &f!("pub/{name}.wasm"),
    )
    .unwrap();
    // moving lib types into the crate for development convenience
    rename(
        &f!("target/{name}/{WASM_UNK}/{PROFILE}/{name}.d.ts"),
        &f!("{name}/lib.d.ts"),
    )
    .unwrap();

    // moving bindings into the crate to include in the bundle
    rename(
        &f!("target/{name}/{WASM_UNK}/{PROFILE}/{name}.js"),
        &f!("{name}/lib.js"),
    )
    .unwrap();
    // bundle and transpile main.ts
    let js = swc::run(name, PROFILE == "release", true).unwrap();
    write(f!("pub/{name}.js"), &js).unwrap();

    // clean up lib.js bindings
    remove_file(&f!("{name}/lib.js")).unwrap();

    bench(
        &f!("{name} compiled into wasm and bundled with typescript"),
        start,
    );
}

pub fn build_wasi(name: &str) {
    let start = Instant::now();
    cargo::build(name, Some(WASM_WASI));
    rename(
        &f!("target/{name}/{WASM_WASI}/{PROFILE}/{name}.wasm"),
        &f!("{name}.wasm"),
    )
    .unwrap();
    bench(&f!("{name} built as a wasi binary"), start);
}

pub fn build(name: &str) {
    let start = Instant::now();
    cargo::build(name, None);
    bench(&f!("{name} built for the local platform"), start);
}

fn bench(message: &str, start: Instant) {
    println!(
        "cargo:warning={}",
        format!("{} in {}ms", message, start.elapsed().as_millis())
    );
}

#[allow(dead_code)]
pub(crate) fn debug(message: &str) {
    println!("cargo:warning=(debug): {message}");
}
