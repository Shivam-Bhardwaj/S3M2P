// Wave simulation for Chladni patterns
// Implements 2D wave equation with modal excitation

use glam::Vec2;
use crate::PlateMode;

/// Chladni eigenmode patterns
#[derive(Clone, Copy, Debug)]
pub enum ChladniMode {
    /// Square plate with fixed edges
    SquareFixed,
    /// Square plate with free edges
    SquareFree,
    /// Circular plate
    Circular,
}

/// 2D Wave simulation on a grid
pub struct WaveSimulation {
    pub width: usize,
    pub height: usize,
    pub amplitude: Vec<f32>,    // Current wave height
    pub velocity: Vec<f32>,     // Rate of change
    pub energy: Vec<f32>,       // Energy density for visualization
}

impl WaveSimulation {
    pub fn new(size: usize) -> Self {
        let len = size * size;
        Self {
            width: size,
            height: size,
            amplitude: vec![0.0; len],
            velocity: vec![0.0; len],
            energy: vec![0.0; len],
        }
    }

    /// Update wave field for one timestep
    pub fn update(&mut self, dt: f32, mode: PlateMode, wave_speed: f32) {
        let w = self.width;
        let h = self.height;

        // Calculate Chladni pattern amplitude
        // For a square plate: A_mn(x,y) = sin(m*pi*x/L) * sin(n*pi*y/L)
        let pi = std::f32::consts::PI;
        let m = mode.m as f32;
        let n = mode.n as f32;

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

                // Superposition creates complex Chladni figures
                self.amplitude[idx] = mode1 + mode2;

                // Energy is proportional to amplitude squared
                self.energy[idx] = self.amplitude[idx].powi(2);
            }
        }
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

    /// Get gradient of wave amplitude (for particle movement)
    pub fn gradient_at(&self, x: f32, y: f32) -> Vec2 {
        let eps = 1.0;

        let ax_pos = self.amplitude_at(x + eps, y);
        let ax_neg = self.amplitude_at(x - eps, y);
        let ay_pos = self.amplitude_at(x, y + eps);
        let ay_neg = self.amplitude_at(x, y - eps);

        // Gradient of amplitude squared (particles move to minima)
        let a_center = self.amplitude_at(x, y);
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
}
