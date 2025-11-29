// Portfolio - Interactive robotics visualization
// Demonstrates: Boid flocking, EKF filtering, A* pathfinding
#![allow(unexpected_cfgs)]

use wasm_bindgen::prelude::*;

/// WASM entry point
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"Portfolio initialized! Full implementation via issues.".into());
    Ok(())
}

// Demo modules - to be implemented via issues
// pub mod demos;
// pub mod render;
