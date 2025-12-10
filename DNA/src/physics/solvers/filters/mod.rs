//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: mod.rs | DNA/src/physics/solvers/filters/mod.rs
//! PURPOSE: Module exports: ekf
//! MODIFIED: 2025-12-09
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════

//!
//! Filters for state estimation from noisy measurements:
//! - ekf.rs     - Extended Kalman Filter (2D position/velocity)
//! - (future)   - Particle filter, UKF, complementary filter
//!
//! ═══════════════════════════════════════════════════════════════════════════════

pub mod ekf;
pub use ekf::{smooth_trajectory, EKF};
