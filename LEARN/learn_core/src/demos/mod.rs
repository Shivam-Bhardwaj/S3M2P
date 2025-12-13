//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: mod.rs | LEARN/learn_core/src/demos/mod.rs
//! PURPOSE: Demo implementations for all LEARN apps
//! MODIFIED: 2025-12-11
//! LAYER: LEARN → learn_core → demos
//! ═══════════════════════════════════════════════════════════════════════════════

pub mod linear_regression;
pub mod particle_filter;
pub mod gpio_debounce;
pub mod fs_permissions;

pub use linear_regression::LinearRegressionDemo;
pub use particle_filter::ParticleFilterDemo;
pub use gpio_debounce::GpioDebounceDemo;
pub use fs_permissions::FsPermissionsDemo;
