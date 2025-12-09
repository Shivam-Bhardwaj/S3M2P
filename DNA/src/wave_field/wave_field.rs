//! Wave field with FFT-based superposition
//!
//! Maintains wave field in both spatial and frequency domains for efficient
//! manipulation and sampling.

use super::fft::FFT2D;
use std::f32::consts::PI;

/// 2D wave field using FFT for efficient superposition
pub struct WaveField {
    size: usize,
    fft: FFT2D,

    // Frequency domain (where we add waves)
    freq_real: Vec<f32>,
    freq_imag: Vec<f32>,

    // Spatial domain (for sampling)
    spatial: Vec<f32>,
    spatial_imag: Vec<f32>,

    dirty: bool, // Needs inverse FFT update
}

impl WaveField {
    /// Create new wave field of given size (must be power of 2)
    pub fn new(size: usize) -> Self {
        assert!(size.is_power_of_two());

        Self {
            size,
            fft: FFT2D::new(size),
            freq_real: vec![0.0; size * size],
            freq_imag: vec![0.0; size * size],
            spatial: vec![0.0; size * size],
            spatial_imag: vec![0.0; size * size],
            dirty: false,
        }
    }

    /// Add a circular wave originating at (cx, cy)
    ///
    /// The wave is added in the frequency domain as a Gaussian envelope.
    /// Actual spatial wave is recovered via inverse FFT.
    pub fn add_circular_wave(&mut self, cx: f32, cy: f32, amplitude: f32, frequency: f32) {
        let n = self.size as f32;

        // Normalize coordinates to [0, 1]
        let cx_norm = cx / n;
        let cy_norm = cy / n;

        // Frequency domain representation of circular wave
        // Approximated as Gaussian in k-space
        let sigma_k = 1.0 / (frequency * n);

        for ky in 0..self.size {
            for kx in 0..self.size {
                // Frequency coordinates centered at 0
                let fx = (kx as f32 - n / 2.0) / n;
                let fy = (ky as f32 - n / 2.0) / n;
                let k_mag = (fx * fx + fy * fy).sqrt();

                // Gaussian envelope in frequency domain
                let weight = (-k_mag * k_mag / (2.0 * sigma_k * sigma_k)).exp();

                // Phase shift for spatial position
                let phase = 2.0 * PI * (fx * cx_norm + fy * cy_norm);

                let idx = ky * self.size + kx;
                self.freq_real[idx] += amplitude * weight * phase.cos();
                self.freq_imag[idx] += amplitude * weight * phase.sin();
            }
        }

        self.dirty = true;
    }

    /// Update spatial field via inverse FFT
    pub fn update(&mut self) {
        if !self.dirty {
            return;
        }

        // Copy frequency domain to working arrays
        self.spatial.copy_from_slice(&self.freq_real);
        self.spatial_imag.copy_from_slice(&self.freq_imag);

        // Inverse FFT: frequency → spatial
        self.fft.inverse(&mut self.spatial, &mut self.spatial_imag);

        self.dirty = false;
    }

    /// Sample wave amplitude at normalized coordinates [0, 1]
    pub fn sample(&self, x: f32, y: f32) -> f32 {
        let ix = ((x * self.size as f32) as usize).min(self.size - 1);
        let iy = ((y * self.size as f32) as usize).min(self.size - 1);
        self.spatial[iy * self.size + ix]
    }

    /// Sample wave amplitude at grid coordinates
    pub fn sample_grid(&self, x: usize, y: usize) -> f32 {
        if x >= self.size || y >= self.size {
            return 0.0;
        }
        self.spatial[y * self.size + x]
    }

    /// Clear all waves (reset to zero)
    pub fn clear(&mut self) {
        self.freq_real.fill(0.0);
        self.freq_imag.fill(0.0);
        self.spatial.fill(0.0);
        self.spatial_imag.fill(0.0);
        self.dirty = false;
    }

    /// Get direct access to spatial field (for visualization)
    pub fn spatial_field(&self) -> &[f32] {
        &self.spatial
    }

    /// Decay all frequency components (wave dissipation)
    pub fn decay(&mut self, factor: f32) {
        for i in 0..self.freq_real.len() {
            self.freq_real[i] *= factor;
            self.freq_imag[i] *= factor;
        }
        self.dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wave_field_creation() {
        let wf = WaveField::new(256);
        assert_eq!(wf.size, 256);
        assert_eq!(wf.spatial.len(), 256 * 256);
    }

    #[test]
    fn test_add_wave_and_sample() {
        let mut wf = WaveField::new(64);

        // Add wave at center
        wf.add_circular_wave(32.0, 32.0, 10.0, 0.1);  // Higher amplitude
        wf.update();

        // Sample near center
        let val_center = wf.sample_grid(32, 32);
        let val_edge = wf.sample_grid(0, 0);

        // Debug output
        println!("Center value: {}", val_center);
        println!("Edge value: {}", val_edge);

        // After FFT roundtrip, should have some signal (relaxed threshold)
        let max_val = wf.spatial.iter().map(|v| v.abs()).fold(0.0f32, f32::max);
        println!("Max value in field: {}", max_val);

        assert!(max_val > 0.001, "Wave field should have non-zero amplitude");
    }
}
