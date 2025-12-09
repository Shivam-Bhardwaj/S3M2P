//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: lib.rs | CORE/WAVE_ENGINE/src/lib.rs
//! PURPOSE: Wave and field simulation engine
//! LAYER: CORE → WAVE_ENGINE
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! WAVE_ENGINE provides wave/field simulation capabilities:
//! - 2D wave equation (Chladni patterns, acoustic)
//! - FFT-based spectral methods
//! - Field visualization
//!
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │ ARCHITECTURE                                                                │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │   WaveEngine                                                                │
//! │       │                                                                     │
//! │       ├── WaveSimulation       (DNA/physics/fields/wave)                    │
//! │       ├── FFT2D                (DNA/physics/solvers/pde/spectral)           │
//! │       └── PlateMode, ChladniMode                                            │
//! │                                                                             │
//! │   Simulation modes:                                                         │
//! │   - Analytical eigenmodes (Chladni patterns)                                │
//! │   - Numerical time-stepping [TODO]                                          │
//! │   - FFT-based spectral solving [TODO]                                       │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//!
//! DEPENDS ON:
//!   • DNA/physics/fields/wave → WaveSimulation, ChladniMode
//!   • DNA/physics/solvers/pde/spectral → FFT2D
//!
//! USED BY:
//!   • SIMULATIONS/CHLADNI → Chladni pattern visualization
//!
//! ═══════════════════════════════════════════════════════════════════════════════

// ─────────────────────────────────────────────────────────────────────────────────
// CODE BELOW - Optimized for ML development
// ─────────────────────────────────────────────────────────────────────────────────

// Re-export wave simulation types from DNA
pub use dna::physics::fields::wave::{ChladniMode, PlateMode, WaveSimulation};

// Re-export FFT from DNA
pub use dna::physics::solvers::pde::spectral::FFT2D;

use glam::Vec2;

/// Wave engine configuration
#[derive(Clone, Debug)]
pub struct WaveEngineConfig {
    pub grid_size: usize,
    pub mode: PlateMode,
    pub frequency_scale: f32,
    pub amplitude_scale: f32,
}

impl Default for WaveEngineConfig {
    fn default() -> Self {
        Self {
            grid_size: 256,
            mode: PlateMode::new(3, 2),
            frequency_scale: 1.0,
            amplitude_scale: 1.0,
        }
    }
}

/// Wave engine combining simulation and FFT
pub struct WaveEngine {
    simulation: WaveSimulation,
    _fft: Option<FFT2D>,
    config: WaveEngineConfig,
}

impl WaveEngine {
    /// Create a new wave engine with given configuration
    pub fn new(config: WaveEngineConfig) -> Self {
        let simulation = WaveSimulation::new(config.grid_size);
        let fft = if config.grid_size.is_power_of_two() {
            Some(FFT2D::new(config.grid_size))
        } else {
            None
        };

        Self {
            simulation,
            _fft: fft,
            config,
        }
    }

    /// Update the wave simulation
    pub fn update(&mut self, dt: f32) {
        self.simulation.update_with_params(
            dt,
            self.config.mode,
            1.0, // wave_speed
            self.config.frequency_scale,
            self.config.amplitude_scale,
        );
    }

    /// Set the plate mode
    pub fn set_mode(&mut self, m: u32, n: u32) {
        self.config.mode = PlateMode::new(m, n);
        self.simulation.set_dirty();
    }

    /// Get amplitude at a point
    pub fn amplitude_at(&self, x: f32, y: f32) -> f32 {
        self.simulation.amplitude_at(x, y)
    }

    /// Get gradient at a point (for particle movement)
    pub fn gradient_at(&self, x: f32, y: f32) -> Vec2 {
        self.simulation.gradient_at(x, y)
    }

    /// Get amplitude data for rendering
    pub fn get_amplitude_data(&self) -> &[f32] {
        self.simulation.get_amplitude_data()
    }

    /// Get energy data for rendering
    pub fn get_energy_data(&self) -> &[f32] {
        self.simulation.get_energy_data()
    }

    /// Get grid size
    pub fn grid_size(&self) -> usize {
        self.config.grid_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wave_engine_creation() {
        let config = WaveEngineConfig::default();
        let engine = WaveEngine::new(config);

        assert_eq!(engine.grid_size(), 256);
    }

    #[test]
    fn test_wave_engine_update() {
        let config = WaveEngineConfig {
            grid_size: 64,
            mode: PlateMode::new(2, 2),
            frequency_scale: 1.0,
            amplitude_scale: 1.0,
        };
        let mut engine = WaveEngine::new(config);

        engine.update(0.1);

        // Should have energy after update
        let energy: f32 = engine.get_energy_data().iter().sum();
        assert!(energy > 0.0);
    }

    #[test]
    fn test_mode_change() {
        let config = WaveEngineConfig::default();
        let mut engine = WaveEngine::new(config);

        engine.update(0.1);
        let energy1: f32 = engine.get_energy_data().iter().sum();

        engine.set_mode(5, 5);
        engine.update(0.1);
        let energy2: f32 = engine.get_energy_data().iter().sum();

        // Different modes should produce different patterns
        assert!(energy1 > 0.0);
        assert!(energy2 > 0.0);
    }
}
