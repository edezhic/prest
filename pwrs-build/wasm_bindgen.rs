use crate::{WASM_UNK, PROFILE};
use anyhow::Error;
use std::format as f;

use wasm_bindgen_cli_support::Bindgen;

pub fn run() -> Result<(), Error> {
    let mut bindgen = Bindgen::new();
    bindgen
        .input_path(f!("target/sw/{WASM_UNK}/{PROFILE}/lib.wasm"))
        .web(true)?
        .typescript(true)
        .remove_name_section(true)
        .remove_producers_section(true)
        .omit_default_module_path(true);
    bindgen.generate(f!("target/sw/{WASM_UNK}/{PROFILE}"))?;
    Ok(())
}
