//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: lib.rs | LEARN/learn_web/src/lib.rs
//! PURPOSE: WASM utilities for LEARN apps
//! MODIFIED: 2025-12-11
//! LAYER: LEARN → learn_web
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! This crate provides:
//! - Hi-DPI canvas wrapper
//! - Animation loop with requestAnimationFrame
//! - Control binding utilities
//! - Hash routing helpers
//! - SimRunner for managing demo state

pub mod animation;
pub mod canvas;
pub mod controls;
pub mod dom;
pub mod routing;

pub use animation::AnimationLoop;
pub use canvas::Canvas;
pub use controls::{wire_button, wire_range};
pub use dom::{get_element, get_element_by_id, set_text};
pub use routing::{get_current_route, navigate_to, setup_routing, Route};

// Re-export learn_core types commonly used with learn_web
pub use learn_core::{Demo, ParamMeta, Rng, Vec2};

use wasm_bindgen::JsValue;

/// Initialize panic hook for better error messages in console
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Log a message to the browser console
pub fn log(msg: &str) {
    web_sys::console::log_1(&JsValue::from_str(msg));
}

/// Log an error to the browser console
pub fn error(msg: &str) {
    web_sys::console::error_1(&JsValue::from_str(msg));
}
