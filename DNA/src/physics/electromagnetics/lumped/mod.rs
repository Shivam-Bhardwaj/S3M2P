//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: mod.rs | DNA/src/physics/electromagnetics/lumped/mod.rs
//! PURPOSE: Module exports: netlist, matrix, ac
//! MODIFIED: 2025-12-09
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════

//!
//! Lumped circuit analysis using Modified Nodal Analysis (MNA):
//! - netlist.rs  - Circuit element definitions and netlist representation
//! - matrix.rs   - Real-valued MNA matrix for DC analysis
//! - ac.rs       - Complex MNA matrix for AC/frequency analysis
//!
//! ═══════════════════════════════════════════════════════════════════════════════

pub mod ac;
pub mod matrix;
pub mod netlist;

pub use ac::*;
pub use matrix::*;
pub use netlist::*;
