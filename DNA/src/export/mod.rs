//! Export module for generating PDF and Gerber X2 files
//!
//! This module implements PDF and Gerber generation from scratch,
//! following the CLAUDE.md philosophy of minimizing external dependencies.

pub mod pdf;
pub mod gerber;

pub use pdf::*;
pub use gerber::*;
