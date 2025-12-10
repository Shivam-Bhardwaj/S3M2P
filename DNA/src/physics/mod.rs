//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: mod.rs | DNA/src/physics/mod.rs
//! PURPOSE: Physics simulation root - mechanics, fields, solvers, orbital dynamics
//! MODIFIED: 2025-12-09
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════

//!
//! PHYSICS defines HOW things behave:
//! - core/             - Units, constants, physical quantities
//! - mechanics/        - Particle, rigid body, collision, constraints
//! - fields/           - Scalar, vector, tensor, wave fields
//! - electromagnetics/ - Maxwell, FDTD, lumped circuits
//! - fluids/           - Euler, Navier-Stokes, SPH, Lattice-Boltzmann
//! - thermal/          - Conduction, convection, radiation
//! - orbital/          - Kepler, N-body, perturbation
//! - solvers/          - ODE, PDE, linear, nonlinear solvers
//!
//! ═══════════════════════════════════════════════════════════════════════════════

// ─────────────────────────────────────────────────────────────────────────────────
// SUBMODULES
// ─────────────────────────────────────────────────────────────────────────────────

pub mod core;
pub mod electromagnetics;
pub mod fields;
pub mod fluids;
pub mod mechanics;
pub mod orbital;
pub mod solvers;
pub mod thermal;
