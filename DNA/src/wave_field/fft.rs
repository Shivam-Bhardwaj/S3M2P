//! 2D Fast Fourier Transform (Cooley-Tukey Algorithm)
//!
//! Radix-2 decimation-in-time FFT for square grids (power-of-2 sizes).
//! Used for efficient wave field superposition in quantum simulations.

use std::f32::consts::PI;

/// 2D FFT for real-valued input
/// Computes forward or inverse transform in-place
pub struct FFT2D {
    size: usize,
    twiddle_real: Vec<f32>,
    twiddle_imag: Vec<f32>,
}

impl FFT2D {
    /// Create FFT for square grid of given size (must be power of 2)
    pub fn new(size: usize) -> Self {
        assert!(size.is_power_of_two(), "FFT size must be power of 2");

        // Pre-compute twiddle factors
        let mut twiddle_real = vec![0.0; size / 2];
        let mut twiddle_imag = vec![0.0; size / 2];

        for k in 0..size / 2 {
            let angle = -2.0 * PI * (k as f32) / (size as f32);
            twiddle_real[k] = angle.cos();
            twiddle_imag[k] = angle.sin();
        }

        Self {
            size,
            twiddle_real,
            twiddle_imag,
        }
    }

    /// 1D FFT (Cooley-Tukey decimation-in-time)
    fn fft_1d(
        &self,
        real: &mut [f32],
        imag: &mut [f32],
        stride: usize,
        offset: usize,
        inverse: bool,
    ) {
        let n = self.size;
        if n == 1 {
            return;
        }

        // Bit-reversal permutation
        self.bit_reverse(real, imag, stride, offset);

        // Iterative FFT
        let mut m = 2;
        while m <= n {
            let half_m = m / 2;

            for k in 0..half_m {
                let twiddle_idx = k * n / m;
                let wr = self.twiddle_real[twiddle_idx];
                let mut wi = self.twiddle_imag[twiddle_idx];

                if inverse {
                    wi = -wi;
                }

                let mut j = 0;
                while j < n {
                    let idx1 = offset + (j + k) * stride;
                    let idx2 = offset + (j + k + half_m) * stride;

                    let tr = wr * real[idx2] - wi * imag[idx2];
                    let ti = wr * imag[idx2] + wi * real[idx2];

                    real[idx2] = real[idx1] - tr;
                    imag[idx2] = imag[idx1] - ti;
                    real[idx1] += tr;
                    imag[idx1] += ti;

                    j += m;
                }
            }

            m *= 2;
        }

        // Scaling for inverse
        if inverse {
            let scale = 1.0 / (n as f32);
            for i in 0..n {
                let idx = offset + i * stride;
                real[idx] *= scale;
                imag[idx] *= scale;
            }
        }
    }

    fn bit_reverse(&self, real: &mut [f32], imag: &mut [f32], stride: usize, offset: usize) {
        let n = self.size;
        let log_n = n.trailing_zeros() as usize;

        for i in 0..n {
            // Reverse only log_n bits (not all 32/64)
            let mut j = 0usize;
            let mut temp = i;
            for _ in 0..log_n {
                j = (j << 1) | (temp & 1);
                temp >>= 1;
            }

            if j > i {
                let idx_i = offset + i * stride;
                let idx_j = offset + j * stride;
                real.swap(idx_i, idx_j);
                imag.swap(idx_i, idx_j);
            }
        }
    }

    /// Forward 2D FFT (spatial → frequency)
    pub fn forward(&self, real: &mut [f32], imag: &mut [f32]) {
        let n = self.size;

        // FFT each row
        for y in 0..n {
            self.fft_1d(real, imag, 1, y * n, false);
        }

        // FFT each column
        for x in 0..n {
            self.fft_1d(real, imag, n, x, false);
        }
    }

    /// Inverse 2D FFT (frequency → spatial)
    pub fn inverse(&self, real: &mut [f32], imag: &mut [f32]) {
        let n = self.size;

        // Inverse FFT each row
        for y in 0..n {
            self.fft_1d(real, imag, 1, y * n, true);
        }

        // Inverse FFT each column
        for x in 0..n {
            self.fft_1d(real, imag, n, x, true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fft_roundtrip() {
        let size = 8;
        let fft = FFT2D::new(size);

        let mut real = vec![0.0; size * size];
        let mut imag = vec![0.0; size * size];

        // Create simple pattern
        real[0] = 1.0;
        real[size + 1] = 1.0;

        let original_real = real.clone();

        // Forward + Inverse should recover original
        fft.forward(&mut real, &mut imag);
        fft.inverse(&mut real, &mut imag);

        for i in 0..size * size {
            assert!((real[i] - original_real[i]).abs() < 1e-5);
            assert!(imag[i].abs() < 1e-5);
        }
    }
}
