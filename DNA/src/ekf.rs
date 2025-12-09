//! Extended Kalman Filter - DEPRECATED
//!
//! This module is deprecated. Please use `physics::solvers::filters` instead.
//!
//! # Migration
//!
//! Old path:
//! ```ignore
//! use dna::ekf::EKF;
//! use dna::EKF;
//! ```
//!
//! New path:
//! ```ignore
//! use dna::physics::solvers::filters::{EKF, smooth_trajectory};
//! ```

// Re-export EKF from new location for backward compatibility
pub use crate::physics::solvers::filters::EKF;
