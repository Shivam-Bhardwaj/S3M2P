//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: mod.rs | DNA/src/physics/solvers/pde/mod.rs
//! PURPOSE: Partial Differential Equation solvers
//! LAYER: DNA → PHYSICS → SOLVERS → PDE
//! ═══════════════════════════════════════════════════════════════════════════════

/// FFT-based spectral methods (Cooley-Tukey)
pub mod spectral;
pub use spectral::FFT2D;

// pub mod fdm;       // TODO: Finite Difference Method
// pub mod fem;       // TODO: Finite Element Method
