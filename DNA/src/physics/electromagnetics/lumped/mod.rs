//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: mod.rs | DNA/src/physics/electromagnetics/lumped/mod.rs
//! PURPOSE: Lumped circuit element simulation (SPICE-like)
//! LAYER: DNA → PHYSICS → ELECTROMAGNETICS → LUMPED
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! Lumped circuit analysis using Modified Nodal Analysis (MNA):
//! - netlist.rs  - Circuit element definitions and netlist representation
//! - matrix.rs   - Real-valued MNA matrix for DC analysis
//! - ac.rs       - Complex MNA matrix for AC/frequency analysis
//!
//! ═══════════════════════════════════════════════════════════════════════════════

pub mod netlist;
pub mod matrix;
pub mod ac;

pub use netlist::*;
pub use matrix::*;
pub use ac::*;
