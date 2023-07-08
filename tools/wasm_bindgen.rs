use crate::{WASM_UNK, PROFILE};
use anyhow::Error;
use std::format as f;

use wasm_bindgen_cli_support::Bindgen;

pub fn run(name: &str) -> Result<(), Error> {
    let mut bindgen = Bindgen::new();
    bindgen
        .input_path(f!("target/{name}/{WASM_UNK}/{PROFILE}/{name}.wasm"))
        .web(true)?
        .typescript(true)
        .remove_name_section(true)
        .remove_producers_section(true)
        .omit_default_module_path(true)
        .out_name(name);
    bindgen.generate(f!("target/{name}/{WASM_UNK}/{PROFILE}"))?;
    Ok(())
}
