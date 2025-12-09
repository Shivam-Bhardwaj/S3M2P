//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: mod.rs | DNA/src/physics/solvers/filters/mod.rs
//! PURPOSE: State estimation and filtering algorithms
//! LAYER: DNA → PHYSICS → SOLVERS → FILTERS
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! Filters for state estimation from noisy measurements:
//! - ekf.rs     - Extended Kalman Filter (2D position/velocity)
//! - (future)   - Particle filter, UKF, complementary filter
//!
//! ═══════════════════════════════════════════════════════════════════════════════

pub mod ekf;
pub use ekf::{EKF, smooth_trajectory};
