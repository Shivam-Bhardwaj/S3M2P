//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: lib.rs | LEARN/learn_core/src/lib.rs
//! PURPOSE: Pure Rust simulation logic for LEARN apps
//! MODIFIED: 2025-12-11
//! LAYER: LEARN → learn_core
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! This crate provides:
//! - `Demo` trait for simulation logic
//! - `Rng` for deterministic random number generation
//! - `Vec2` and math utilities
//!
//! No web-sys or wasm-bindgen dependencies - fully testable with `cargo test`.

pub mod demo;
pub mod math;
pub mod rng;
pub mod demos;

pub use demo::{Demo, ParamMeta};
pub use math::{clamp, lerp, smoothstep, Vec2};
pub use rng::Rng;
