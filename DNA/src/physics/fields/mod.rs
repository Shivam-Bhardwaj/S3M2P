//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: mod.rs | DNA/src/physics/fields/mod.rs
//! PURPOSE: Field representations - scalar, vector, tensor, wave
//! LAYER: DNA → PHYSICS → FIELDS
//! ═══════════════════════════════════════════════════════════════════════════════

/// Wave equation solver and Chladni patterns
pub mod wave;
pub use wave::{ChladniMode, PlateMode, WaveSimulation};

// pub mod scalar;  // TODO: Temperature, pressure, potential
// pub mod vector;  // TODO: Velocity, force, E/B fields
// pub mod tensor;  // TODO: Stress, strain tensors
