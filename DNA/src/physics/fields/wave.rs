//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: wave.rs
//! PATH: DNA/src/physics/fields/wave.rs
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! PURPOSE: Wave equation solver and Chladni pattern generation
//!
//! LAYER: DNA → PHYSICS → FIELDS
//!
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │ PHYSICS: Wave Equation                                                      │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │ General wave equation:                                                      │
//! │   ∂²u/∂t² = c²∇²u                                                           │
//! │                                                                             │
//! │ For 2D:                                                                     │
//! │   ∂²u/∂t² = c²(∂²u/∂x² + ∂²u/∂y²)                                           │
//! │                                                                             │
//! │ Chladni eigenmode (square plate):                                           │
//! │   A_mn(x,y) = sin(m·π·x/L) · sin(n·π·y/L)                                   │
//! │                                                                             │
//! │ Frequency:                                                                  │
//! │   f_mn = C · (m² + n²)    where C depends on plate properties               │
//! └─────────────────────────────────────────────────────────────────────────────┘
//!
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │ DATA DEFINED                                                                │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │ ChladniMode         Boundary condition type (fixed, free, circular)         │
//! │ PlateMode           Mode numbers (m, n) for eigenmode selection             │
//! │ WaveSimulation      2D wave field state (amplitude, velocity, energy)       │
//! └─────────────────────────────────────────────────────────────────────────────┘
//!
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │ DATA FLOW                                                                   │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │ CONSUMES:  PlateMode, f32 (dt, wave_speed, scale parameters)                │
//! │ PRODUCES:  f32[] (amplitude), f32[] (energy), Vec2 (gradient)               │
//! └─────────────────────────────────────────────────────────────────────────────┘
//!
//! DEPENDS ON:
//!   • glam::Vec2 → Gradient calculation
//!
//! USED BY:
//!   • SIMULATIONS/CHLADNI → Chladni pattern visualization
//!   • Particle systems → Particles moving to nodal lines
//!
//! ═══════════════════════════════════════════════════════════════════════════════

// ─────────────────────────────────────────────────────────────────────────────────
// CODE BELOW - Optimized for ML development
// ─────────────────────────────────────────────────────────────────────────────────

use glam::Vec2;

/// Chladni eigenmode patterns (boundary conditions)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChladniMode {
    /// Square plate with fixed edges
    SquareFixed,
    /// Square plate with free edges
    SquareFree,
    /// Circular plate
    Circular,
}

/// Chladni plate modes (m, n) - defines the vibration pattern
#[derive(Clone, Copy, Debug)]
pub struct PlateMode {
    /// Horizontal mode number
    pub m: u32,
    /// Vertical mode number
    pub n: u32,
}

impl PlateMode {
    pub fn new(m: u32, n: u32) -> Self {
        Self { m, n }
    }

    /// Calculate frequency for a square plate
    /// f_mn = C * (m² + n²) where C depends on plate properties
    pub fn frequency(&self, plate_constant: f32) -> f32 {
        plate_constant * ((self.m * self.m + self.n * self.n) as f32)
    }
}

/// 2D Wave simulation on a grid
pub struct WaveSimulation {
    pub width: usize,
    pub height: usize,
    /// Current wave height at each grid point
    pub amplitude: Vec<f32>,
    /// Rate of change of amplitude
    pub velocity: Vec<f32>,
    /// Energy density for visualization (amplitude²)
    pub energy: Vec<f32>,
    /// Optimization: skip updates if params haven't changed
    dirty: bool,
}

impl WaveSimulation {
    /// Create a new wave simulation with given grid size
    pub fn new(size: usize) -> Self {
        let len = size * size;
        Self {
            width: size,
            height: size,
            amplitude: vec![0.0; len],
            velocity: vec![0.0; len],
            energy: vec![0.0; len],
            dirty: true,
        }
    }

    /// Force recomputation on next update
    pub fn set_dirty(&mut self) {
        self.dirty = true;
    }

    /// Check if simulation needs update
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Update wave field for one timestep (simplified interface)
    pub fn update(&mut self, dt: f32, mode: PlateMode, wave_speed: f32) {
        self.update_with_params(dt, mode, wave_speed, 1.0, 1.0);
    }

    /// Update wave field with frequency scale and amplitude parameters
    ///
    /// This implementation uses analytical standing wave solutions (Chladni eigenmodes)
    /// rather than numerical time-stepping, making it suitable for visualization.
    pub fn update_with_params(
        &mut self,
        _dt: f32,
        mode: PlateMode,
        _wave_speed: f32,
        frequency_scale: f32,
        amplitude_scale: f32,
    ) {
        // Optimization: Skip if nothing changed
        if !self.dirty {
            return;
        }

        let w = self.width;
        let h = self.height;

        // Calculate Chladni pattern amplitude
        // For a square plate: A_mn(x,y) = sin(m*π*x/L) * sin(n*π*y/L)
        let pi = std::f32::consts::PI;
        // Apply frequency scale to mode numbers for pattern complexity
        let m = mode.m as f32 * frequency_scale;
        let n = mode.n as f32 * frequency_scale;

        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;

                // Normalized coordinates [0, 1]
                let nx = x as f32 / w as f32;
                let ny = y as f32 / h as f32;

                // Chladni eigenmode (standing wave pattern)
                // Using combination of modes for interesting patterns
                let mode1 = (m * pi * nx).sin() * (n * pi * ny).sin();
                let mode2 = (n * pi * nx).sin() * (m * pi * ny).sin();

                // Superposition creates complex Chladni figures, scaled by amplitude
                self.amplitude[idx] = (mode1 + mode2) * amplitude_scale;

                // Energy is proportional to amplitude squared
                self.energy[idx] = self.amplitude[idx].powi(2);
            }
        }

        self.dirty = false;
    }

    /// Get wave amplitude at a point (bilinear interpolation)
    pub fn amplitude_at(&self, x: f32, y: f32) -> f32 {
        let w = self.width;
        let h = self.height;

        let x = x.clamp(0.0, (w - 1) as f32);
        let y = y.clamp(0.0, (h - 1) as f32);

        let x0 = x.floor() as usize;
        let y0 = y.floor() as usize;
        let x1 = (x0 + 1).min(w - 1);
        let y1 = (y0 + 1).min(h - 1);

        let fx = x.fract();
        let fy = y.fract();

        let a00 = self.amplitude[y0 * w + x0];
        let a10 = self.amplitude[y0 * w + x1];
        let a01 = self.amplitude[y1 * w + x0];
        let a11 = self.amplitude[y1 * w + x1];

        let a0 = a00 * (1.0 - fx) + a10 * fx;
        let a1 = a01 * (1.0 - fx) + a11 * fx;

        a0 * (1.0 - fy) + a1 * fy
    }

    /// Get gradient of wave amplitude (for particle movement toward nodal lines)
    pub fn gradient_at(&self, x: f32, y: f32) -> Vec2 {
        let eps = 1.0;

        let ax_pos = self.amplitude_at(x + eps, y);
        let ax_neg = self.amplitude_at(x - eps, y);
        let ay_pos = self.amplitude_at(x, y + eps);
        let ay_neg = self.amplitude_at(x, y - eps);

        // Gradient of amplitude squared (particles move to minima)
        let dx = (ax_pos.powi(2) - ax_neg.powi(2)) / (2.0 * eps);
        let dy = (ay_pos.powi(2) - ay_neg.powi(2)) / (2.0 * eps);

        Vec2::new(dx, dy)
    }

    /// Get amplitude data for rendering
    pub fn get_amplitude_data(&self) -> &[f32] {
        &self.amplitude
    }

    /// Get energy data for rendering
    pub fn get_energy_data(&self) -> &[f32] {
        &self.energy
    }

    /// Get total energy in the system
    pub fn total_energy(&self) -> f32 {
        self.energy.iter().sum()
    }

    /// Clear the simulation to zero state
    pub fn clear(&mut self) {
        self.amplitude.fill(0.0);
        self.velocity.fill(0.0);
        self.energy.fill(0.0);
        self.dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plate_mode_frequency() {
        let mode = PlateMode::new(1, 1);
        let constant = 1.0;
        // f = C * (m² + n²) = 1 * (1 + 1) = 2
        assert_eq!(mode.frequency(constant), 2.0);

        let mode2 = PlateMode::new(2, 3);
        // f = 1 * (4 + 9) = 13
        assert_eq!(mode2.frequency(constant), 13.0);
    }

    #[test]
    fn test_simulation_initialization() {
        let size = 10;
        let sim = WaveSimulation::new(size);
        assert_eq!(sim.width, size);
        assert_eq!(sim.height, size);
        assert_eq!(sim.amplitude.len(), size * size);
        assert!(sim.amplitude.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_amplitude_at_bounds() {
        let mut sim = WaveSimulation::new(10);
        // Set a known value at (5, 5)
        let idx = 5 * 10 + 5;
        sim.amplitude[idx] = 1.0;

        // Exact hit
        assert_eq!(sim.amplitude_at(5.0, 5.0), 1.0);

        // Out of bounds should clamp
        assert_eq!(sim.amplitude_at(-1.0, -1.0), sim.amplitude_at(0.0, 0.0));
        assert_eq!(sim.amplitude_at(100.0, 100.0), sim.amplitude_at(9.0, 9.0));
    }

    #[test]
    fn test_update_modifies_state() {
        let mut sim = WaveSimulation::new(20);
        let mode = PlateMode::new(1, 1);

        // Initial state is zero
        assert!(sim.energy.iter().all(|&x| x == 0.0));

        // Update
        sim.update(0.1, mode, 1.0);

        // Should have some energy now
        let total_energy: f32 = sim.energy.iter().sum();
        assert!(total_energy > 0.0);
    }

    #[test]
    fn test_dirty_flag_optimization() {
        let mut sim = WaveSimulation::new(20);
        let mode = PlateMode::new(1, 1);

        // 1. First update (dirty is true by default)
        sim.update(0.1, mode, 1.0);
        let initial_energy: f32 = sim.energy.iter().sum();
        assert!(initial_energy > 0.0);

        // 2. Tamper with data to prove skipping
        sim.energy.fill(0.0);

        // 3. Update again (should count as clean, so NO computation)
        sim.update(0.1, mode, 1.0);
        let skipped_energy: f32 = sim.energy.iter().sum();
        assert_eq!(
            skipped_energy, 0.0,
            "Should skip computation (energy stays 0) when not dirty"
        );

        // 4. Force dirty
        sim.set_dirty();

        // 5. Update again (should recompute)
        sim.update(0.1, mode, 1.0);
        let recomputed_energy: f32 = sim.energy.iter().sum();
        assert!(
            (recomputed_energy - initial_energy).abs() < 0.001,
            "Should restore original values"
        );
    }

    #[test]
    fn test_total_energy() {
        let mut sim = WaveSimulation::new(10);
        let mode = PlateMode::new(2, 2);
        sim.update(0.1, mode, 1.0);

        let total = sim.total_energy();
        let manual_sum: f32 = sim.energy.iter().sum();
        assert_eq!(total, manual_sum);
    }

    #[test]
    fn test_clear() {
        let mut sim = WaveSimulation::new(10);
        let mode = PlateMode::new(1, 1);
        sim.update(0.1, mode, 1.0);

        sim.clear();

        assert!(sim.amplitude.iter().all(|&x| x == 0.0));
        assert!(sim.energy.iter().all(|&x| x == 0.0));
        assert!(sim.is_dirty());
    }
}
