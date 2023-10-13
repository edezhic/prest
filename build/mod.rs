#[cfg(feature = "typescript")]
mod typescript;
#[cfg(feature = "typescript")]
pub use typescript::bundle_ts;
#[cfg(feature = "pwa")]
mod pwa;
#[cfg(feature = "pwa")]
pub use pwa::generate_pwa_assets;
#[cfg(feature = "sass")]
mod sass;
#[cfg(feature = "sass")]
pub use sass::bundle_sass;

use std::{
    fs::{create_dir_all, read_to_string, rename, write, copy},
    path::Path,
    process::Command,
    time::Instant,
};
use once_cell::sync::Lazy;


pub static WASM_UNK: &str = "wasm32-unknown-unknown";
pub static WASM_WASI: &str = "wasm32-wasi";
pub static SW_TARGET_DIR: &str = "target_sw";
#[cfg(debug_assertions)]
pub static PROFILE: &str = "debug";
#[cfg(not(debug_assertions))]
pub static PROFILE: &str = "release";
pub static ASSETS_DIR: Lazy<String> = Lazy::new(|| {
    let dir = format!("{}/assets", std::env::var("OUT_DIR").unwrap());
    create_dir_all(&dir).unwrap();
    dir
});

pub fn include_asset(path: &str) {
    let path = Path::new(path);
    let filename = path.file_name().unwrap();
    std::fs::copy(path, format!("{}/{}", *ASSETS_DIR, filename.to_str().unwrap())).unwrap();
}

pub fn out_path(filename: &str) -> String {
    format!("{}/{filename}", *ASSETS_DIR)
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
