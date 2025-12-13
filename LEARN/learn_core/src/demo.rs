//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: demo.rs | LEARN/learn_core/src/demo.rs
//! PURPOSE: Demo trait and parameter metadata for interactive simulations
//! MODIFIED: 2025-12-11
//! LAYER: LEARN → learn_core
//! ═══════════════════════════════════════════════════════════════════════════════

/// Metadata for a simulation parameter exposed to the UI
#[derive(Clone, Copy, Debug)]
pub struct ParamMeta {
    /// Internal name (used in set_param)
    pub name: &'static str,
    /// Display label for UI
    pub label: &'static str,
    /// Minimum value
    pub min: f32,
    /// Maximum value
    pub max: f32,
    /// Step increment for slider
    pub step: f32,
    /// Default value
    pub default: f32,
}

/// Trait for interactive simulation demos
///
/// Implementations should be pure Rust with no web dependencies.
/// Rendering is handled separately by `DemoRenderer` in learn_web.
pub trait Demo: Default {
    /// Reset the simulation with a seed for reproducibility
    fn reset(&mut self, seed: u64);

    /// Advance the simulation by dt seconds
    fn step(&mut self, dt: f32);

    /// Set a named parameter, returns true if parameter exists
    fn set_param(&mut self, name: &str, value: f32) -> bool;

    /// Get metadata for all tunable parameters
    fn params() -> &'static [ParamMeta];
}
