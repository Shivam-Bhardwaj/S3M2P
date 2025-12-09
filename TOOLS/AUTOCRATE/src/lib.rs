// AutoCrate - ASTM Standard Shipping Crate Generator
// Rust/WASM port of the original TypeScript application
#![allow(unexpected_cfgs)]

use wasm_bindgen::prelude::*;

pub use autocrate_engine::*;

/// WASM entry point
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"AutoCrate initialized! Full implementation via issues.".into());
    Ok(())
}
