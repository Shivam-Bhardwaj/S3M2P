use super::types::{BodePlot, PLLPerformance};
use std::f64::consts::PI;

/// Complex number for AC analysis
#[derive(Clone, Copy, Debug)]
struct Complex {
    real: f64,
    imag: f64,
}

impl Complex {
    fn new(real: f64, imag: f64) -> Self {
        Self { real, imag }
    }

    fn magnitude(&self) -> f64 {
        (self.real * self.real + self.imag * self.imag).sqrt()
    }

    fn phase_deg(&self) -> f64 {
        self.imag.atan2(self.real) * 180.0 / PI
    }

    fn mul(&self, other: &Complex) -> Complex {
        Complex {
            real: self.real * other.real - self.imag * other.imag,
            imag: self.real * other.imag + self.imag * other.real,
        }
    }

    fn div(&self, other: &Complex) -> Complex {
        let denom = other.real * other.real + other.imag * other.imag;
        Complex {
            real: (self.real * other.real + self.imag * other.imag) / denom,
            imag: (self.imag * other.real - self.real * other.imag) / denom,
        }
    }
}

/// Evaluate loop filter impedance Z(s) at s = jω
///
/// For 2nd order: Z(s) = R1 + 1/(sC1) || 1/(sC2)
/// Simplified: Z(s) = (1 + sR1C1) / (sC1(1 + sR1C2))
fn eval_loop_filter_impedance(
    omega: f64,
    r1: f64,
    c1: f64,
    c2: f64,
) -> Complex {
    let s_imag = omega;

    // Numerator: 1 + jωR1C1
    let num_real = 1.0;
    let num_imag = s_imag * r1 * c1;

    // Denominator: jωC1(1 + jωR1C2)
    // = jωC1 + jω(jω)R1C1C2
    // = jωC1 - ω²R1C1C2
    let denom_real = -s_imag * s_imag * r1 * c1 * c2;
    let denom_imag = s_imag * c1;

    let num = Complex::new(num_real, num_imag);
    let denom = Complex::new(denom_real, denom_imag);

    num.div(&denom)
}

/// Calculate open-loop transfer function G(s)H(s)
///
/// G(s)H(s) = (K_phi * K_vco / N) * Z(s) / s
fn eval_open_loop_gain(
    omega: f64,
    k_phi: f64,
    k_vco: f64,
    n: f64,
    r1: f64,
    c1: f64,
    c2: f64,
) -> Complex {
    // DC gain
    let k = (k_phi * k_vco) / n;

    // Z(s)
    let z = eval_loop_filter_impedance(omega, r1, c1, c2);

    // 1/s = -j/ω
    let one_over_s = Complex::new(0.0, -1.0 / omega);

    // Multiply: K * Z(s) * (1/s)
    let gain_times_z = Complex::new(k * z.real, k * z.imag);
    gain_times_z.mul(&one_over_s)
}

/// Generate Bode plot data
pub fn generate_bode_plot(
    k_phi: f64,
    k_vco: f64,
    n: f64,
    r1: f64,
    c1: f64,
    c2: f64,
    freq_start_hz: f64,
    freq_stop_hz: f64,
    points_per_decade: usize,
) -> BodePlot {
    let mut frequencies = Vec::new();
    let mut magnitudes = Vec::new();
    let mut phases = Vec::new();

    let start_log = freq_start_hz.log10();
    let stop_log = freq_stop_hz.log10();
    let decades = stop_log - start_log;
    let num_points = (decades * points_per_decade as f64).ceil() as usize;

    for i in 0..num_points {
        let log_freq = start_log + (i as f64 / (num_points - 1) as f64) * (stop_log - start_log);
        let freq = 10.0_f64.powf(log_freq);
        let omega = 2.0 * PI * freq;

        let g = eval_open_loop_gain(omega, k_phi, k_vco, n, r1, c1, c2);
        let mag_db = 20.0 * g.magnitude().log10();
        let phase_deg = g.phase_deg();

        frequencies.push(freq);
        magnitudes.push(mag_db);
        phases.push(phase_deg);
    }

    BodePlot {
        frequencies_hz: frequencies,
        magnitude_db: magnitudes,
        phase_deg: phases,
    }
}

/// Analyze PLL stability from Bode plot
pub fn analyze_stability(bode: &BodePlot) -> PLLPerformance {
    // Find crossover frequency (where magnitude = 0 dB)
    let mut crossover_freq = 0.0;
    let mut phase_margin = 0.0;

    for i in 0..bode.magnitude_db.len() - 1 {
        if bode.magnitude_db[i] >= 0.0 && bode.magnitude_db[i + 1] < 0.0 {
            // Linear interpolation
            let f1 = bode.frequencies_hz[i];
            let f2 = bode.frequencies_hz[i + 1];
            let m1 = bode.magnitude_db[i];
            let m2 = bode.magnitude_db[i + 1];

            crossover_freq = f1 + (0.0 - m1) * (f2 - f1) / (m2 - m1);

            // Interpolate phase at crossover
            let p1 = bode.phase_deg[i];
            let p2 = bode.phase_deg[i + 1];
            let phase_at_crossover = p1 + (crossover_freq - f1) * (p2 - p1) / (f2 - f1);

            phase_margin = 180.0 + phase_at_crossover;
            break;
        }
    }

    // Find gain margin (magnitude at -180° phase)
    let mut gain_margin = 100.0;  // Default high value if stable

    for i in 0..bode.phase_deg.len() - 1 {
        if bode.phase_deg[i] >= -180.0 && bode.phase_deg[i + 1] < -180.0 {
            // Interpolate magnitude at -180°
            let p1 = bode.phase_deg[i];
            let p2 = bode.phase_deg[i + 1];
            let m1 = bode.magnitude_db[i];
            let m2 = bode.magnitude_db[i + 1];

            let mag_at_180 = m1 + (-180.0 - p1) * (m2 - m1) / (p2 - p1);
            gain_margin = -mag_at_180;  // GM is positive
            break;
        }
    }

    // Estimate lock time (settling time)
    let lock_time_us = if crossover_freq > 0.0 {
        // Approximation: ~5 time constants
        5.0 / (2.0 * PI * crossover_freq) * 1e6
    } else {
        0.0
    };

    PLLPerformance {
        phase_margin_deg: phase_margin,
        gain_margin_db: gain_margin,
        crossover_freq_hz: crossover_freq,
        loop_bandwidth_hz: crossover_freq,  // Approximation
        lock_time_us,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complex_operations() {
        let a = Complex::new(3.0, 4.0);
        assert!((a.magnitude() - 5.0).abs() < 1e-10);

        let b = Complex::new(1.0, 0.0);
        let c = a.mul(&b);
        assert!((c.real - 3.0).abs() < 1e-10);
        assert!((c.imag - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_bode_plot_generation() {
        let k_phi = 1e-3;
        let k_vco = 10e6 * 2.0 * PI;  // Convert to rad/s/V
        let n = 240.0;
        let r1 = 10e3;
        let c1 = 1e-9;
        let c2 = 100e-12;

        let bode = generate_bode_plot(
            k_phi, k_vco, n, r1, c1, c2,
            1e3, 1e7, 20,
        );

        assert!(!bode.frequencies_hz.is_empty());
        assert_eq!(bode.frequencies_hz.len(), bode.magnitude_db.len());
        assert_eq!(bode.frequencies_hz.len(), bode.phase_deg.len());
    }

    #[test]
    fn test_stability_analysis() {
        let k_phi = 1e-3;
        let k_vco = 10e6 * 2.0 * PI;
        let n = 240.0;
        let r1 = 10e3;
        let c1 = 1e-9;
        let c2 = 100e-12;

        let bode = generate_bode_plot(
            k_phi, k_vco, n, r1, c1, c2,
            1e3, 1e7, 50,
        );

        let perf = analyze_stability(&bode);

        assert!(perf.crossover_freq_hz > 0.0);
        assert!(perf.phase_margin_deg > 0.0);
        assert!(perf.phase_margin_deg < 90.0);
        assert!(perf.gain_margin_db > 0.0);
    }
}
