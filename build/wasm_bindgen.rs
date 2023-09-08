use super::{WASM_UNK, PROFILE};
use anyhow::Error;
use std::format as f;

use wasm_bindgen_cli_support::Bindgen;

pub fn run(target_dir: &str, lib_name: &str, types: bool) -> Result<(), Error> {
    let mut bindgen = Bindgen::new();
    bindgen
        .input_path(f!("{target_dir}/{WASM_UNK}/{PROFILE}/{lib_name}.wasm"))
        .web(true)?
        .typescript(types)
        .remove_name_section(true)
        .remove_producers_section(true)
        .omit_default_module_path(true);
    bindgen.generate(f!("{target_dir}/{WASM_UNK}/{PROFILE}"))?;
    Ok(())
}
