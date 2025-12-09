//! Wave field simulation using FFT
//!
//! DNA-level primitive for wave-based simulations. Supports:
//! - Circular wave superposition
//! - FFT-based efficient computation
//! - Wavefunction collapse for particle spawning
//! - Ecosystem simulation with wave-based spawning
//! - Reusable across Chladni, fluid, quantum simulations

mod ecosystem;
mod fft;
#[allow(clippy::module_inception)]
mod wave_field;

pub use ecosystem::{
    analyze_stability, CellType, Ecosystem, FrameMetrics, HyperParams, PIDController,
    StabilityReport, Wave,
};
pub use fft::FFT2D;
pub use wave_field::WaveField;
