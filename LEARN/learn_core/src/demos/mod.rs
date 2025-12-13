//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: mod.rs | LEARN/learn_core/src/demos/mod.rs
//! PURPOSE: Demo implementations for all LEARN apps
//! MODIFIED: 2025-12-12
//! LAYER: LEARN → learn_core → demos
//! ═══════════════════════════════════════════════════════════════════════════════

pub mod linear_regression;
pub mod complementary_filter;
pub mod kalman_filter;
pub mod particle_filter;
pub mod ekf_slam;
pub mod graph_slam;
pub mod gpio_debounce;
pub mod fs_permissions;

pub use linear_regression::LinearRegressionDemo;
pub use complementary_filter::{ComplementaryFilterDemo, ImuReading, SensorHistory};
pub use kalman_filter::{KalmanFilterDemo, KFPhase, Mat2};
pub use particle_filter::{ParticleFilterDemo, PFPhase, Particle, Measurement};
pub use ekf_slam::{EkfSlamDemo, SlamLandmark};
pub use graph_slam::{GraphSlamDemo, PoseNode, GraphEdge};
pub use gpio_debounce::GpioDebounceDemo;
pub use fs_permissions::FsPermissionsDemo;
