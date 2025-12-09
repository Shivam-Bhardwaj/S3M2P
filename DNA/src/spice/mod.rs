//! SPICE circuit simulation - DEPRECATED
//!
//! This module is deprecated. Please use `physics::electromagnetics::lumped` instead.
//!
//! # Migration
//!
//! Old path:
//! ```ignore
//! use dna::spice::{Netlist, Element, ac_analysis};
//! ```
//!
//! New path:
//! ```ignore
//! use dna::physics::electromagnetics::lumped::{Netlist, Element, ac_analysis};
//! ```
//!
//! Or use the CORE engine:
//! ```ignore
//! use spice_engine::{Netlist, Element, ac_analysis};
//! ```

// Re-export all types from new location for backward compatibility
pub use crate::physics::electromagnetics::lumped::*;
