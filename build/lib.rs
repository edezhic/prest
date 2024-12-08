use anyhow::Result;

#[cfg(feature = "pwa")]
mod pwa;
#[cfg(feature = "pwa")]
pub use pwa::*;

#[cfg(feature = "typescript")]
mod typescript;
#[cfg(feature = "typescript")]
pub use typescript::*;

#[cfg(feature = "sass")]
mod sass {
    use std::{fs::write, path::Path};
    pub fn bundle_sass(path: &str) -> anyhow::Result<()> {
        let css = grass::from_path(path, &Default::default())?;
        let scss_filename = Path::new(path).file_name().unwrap().to_str().unwrap();
        let css_filename = scss_filename
            .replace(".scss", ".css")
            .replace(".sass", ".css");
        let out_file = super::out_path(&css_filename);
        write(out_file, css)?;
        Ok(())
    }
}
#[cfg(feature = "sass")]
pub use sass::bundle_sass;

pub use cfg_aliases::cfg_aliases;

pub fn default_cfg_aliases() {
    cfg_aliases! {
        wasm: { target_arch = "wasm32" },
        sw: { wasm },
        not_wasm: { not(wasm) },
        host: { not_wasm },
        debug: { debug_assertions },
        release: { not(debug_assertions) },
    }
}

pub fn read_lib_name() -> Result<String> {
    use toml::{Table, Value};
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let manifest_path = &format!("{manifest_dir}/Cargo.toml");
    let manifest = std::fs::read_to_string(manifest_path)?;
    let parsed = manifest.parse::<Table>()?;
    let lib_name = if parsed.contains_key("lib") {
        let Value::Table(lib_table) = &parsed["lib"] else {
            panic!("should be unreachable");
        };
        if lib_table.contains_key("name") {
            lib_table["name"].as_str().unwrap().to_owned()
        } else {
            parsed["package"]["name"].as_str().unwrap().to_owned()
        }
    } else {
        parsed["package"]["name"].as_str().unwrap().to_owned()
    };
    Ok(lib_name.replace("-", "_"))
}

/// Utility that attempts to find the path of the current build's target path
pub fn find_target_dir() -> Option<String> {
    use std::{ffi::OsStr, path::PathBuf};
    if let Some(target_dir) = std::env::var_os("CARGO_TARGET_DIR") {
        let target_dir = PathBuf::from(target_dir);
        if target_dir.is_absolute() {
            if let Some(str) = target_dir.to_str() {
                return Some(str.to_owned());
            } else {
                return None;
            }
        } else {
            return None;
        };
    }

    let mut dir = PathBuf::from(out_path(""));
    loop {
        if dir.join(".rustc_info.json").exists()
            || dir.join("CACHEDIR.TAG").exists()
            || dir.file_name() == Some(OsStr::new("target"))
                && dir
                    .parent()
                    .map_or(false, |parent| parent.join("Cargo.toml").exists())
        {
            if let Some(str) = dir.to_str() {
                return Some(str.to_owned());
            } else {
                return None;
            }
        }
        if dir.pop() {
            continue;
        }
        return None;
    }
}

/// Utility for composition of paths to build artifacts
pub fn out_path(filename: &str) -> String {
    let dir = std::env::var("OUT_DIR").unwrap();
    format!("{dir}/{filename}")
}
