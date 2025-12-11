//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: mod.rs | DNA/src/export/mod.rs
//! PURPOSE: Module exports: pdf, gerber
//! MODIFIED: 2025-12-02
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════

//! Export module for generating PDF and Gerber X2 files
//!
//! This module implements PDF and Gerber generation from scratch,
//! following the CLAUDE.md philosophy of minimizing external dependencies.

pub mod gerber;
pub mod pdf;
pub mod step;

pub use gerber::*;
pub use pdf::*;
pub use step::*;
