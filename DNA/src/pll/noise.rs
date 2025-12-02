use super::types::{BodePlot, NoiseComponents, PhaseNoiseProfile};
use std::f64::consts::PI;

/// Calculate phase noise profile for the PLL
pub fn calculate_phase_noise(
    bode: &BodePlot,
    n_total: f64,
    ref_freq_hz: f64,
    vco_freq_hz: f64,
) -> PhaseNoiseProfile {
    let mut offsets = Vec::new();
    let mut total_noise = Vec::new();
    let mut components_list = Vec::new();

    // Calculate noise at each frequency point from the Bode plot
    for (i, &freq) in bode.frequencies_hz.iter().enumerate() {
        let loop_gain_mag = 10f64.powf(bode.magnitude_db[i] / 20.0);

        // Transfer functions (approximate)
        // Closed loop transfer function H(s) = G(s) / (1 + G(s)*H_div)
        // For noise transfer:
        // Ref noise: Low pass, gain N
        // VCO noise: High pass
        // PFD/CP/R noise: Low pass, gain N

        // Simplified transfer functions based on loop gain magnitude
        // Inside bandwidth (|G| >> 1): Output tracks reference * N
        // Outside bandwidth (|G| << 1): Output tracks VCO

        let tf_lowpass_sq = (loop_gain_mag * loop_gain_mag) / (1.0 + loop_gain_mag * loop_gain_mag);
        let tf_highpass_sq = 1.0 / (1.0 + loop_gain_mag * loop_gain_mag);

        // 1. Reference Noise Model
        // Typical crystal oscillator: -130 dBc/Hz flat, 1/f flicker below 1kHz
        let ref_noise_floor = 1e-14; // -140 dBc/Hz
        let ref_flicker = 1e-11 / freq; // 1/f corner around 1kHz
        let ref_noise_power = ref_noise_floor + ref_flicker;
        let out_ref_noise = ref_noise_power * n_total * n_total * tf_lowpass_sq;

        // 2. PFD/CP Noise Model
        // Flat noise floor dominated by CP current
        // Normalized to 1Hz: -220 + 10log10(f_pfd) + 20log10(N)
        // Here we model it as input referred noise
        let pfd_noise_floor = 1e-22 * ref_freq_hz; // simplified
        let out_pfd_noise = pfd_noise_floor * n_total * n_total * tf_lowpass_sq;

        // 3. VCO Noise Model (Leeson's Equation)
        // -20dB/dec slope, -30dB/dec flicker
        // Typical VCO: -100 dBc/Hz @ 100kHz offset for 2.4GHz
        let vco_corner = 100e3;
        let vco_noise_at_corner = 1e-10; // -100 dBc/Hz
        let vco_thermal = 1e-16; // -160 dBc/Hz floor

        let vco_1_f2 = vco_noise_at_corner * (vco_corner / freq).powi(2);
        let vco_1_f3 = vco_1_f2 * (10e3 / freq); // Flicker corner 10kHz
        let vco_noise_power = vco_thermal + vco_1_f2 + vco_1_f3;
        let out_vco_noise = vco_noise_power * tf_highpass_sq;

        // 4. Loop Filter Noise (Resistor thermal noise)
        // Simplified: dominated by R1, input referred
        let filter_noise_power = 1e-18; // Very small usually
        let out_filter_noise = filter_noise_power * n_total * n_total * tf_lowpass_sq;

        // 5. Divider Noise
        let div_noise_power = 1e-15; // -150 dBc/Hz
        let out_div_noise = div_noise_power * tf_lowpass_sq; // Appears at output filtered by loop

        // Total Noise Power
        let total_power =
            out_ref_noise + out_pfd_noise + out_vco_noise + out_filter_noise + out_div_noise;

        offsets.push(freq);
        total_noise.push(10.0 * total_power.log10());

        components_list.push(NoiseComponents {
            total_dbc_hz: 10.0 * total_power.log10(),
            ref_dbc_hz: 10.0 * out_ref_noise.log10(),
            pfd_dbc_hz: 10.0 * out_pfd_noise.log10(),
            vco_dbc_hz: 10.0 * out_vco_noise.log10(),
            filter_dbc_hz: 10.0 * out_filter_noise.log10(),
            divider_dbc_hz: 10.0 * out_div_noise.log10(),
        });
    }

    // Calculate Integrated Jitter
    // Integrate L(f) from 1kHz to 100MHz (or max offset)
    let mut jitter_sq = 0.0;
    for i in 0..offsets.len() - 1 {
        let f1 = offsets[i];
        let f2 = offsets[i + 1];
        let l1 = 10f64.powf(total_noise[i] / 10.0);
        let l2 = 10f64.powf(total_noise[i + 1] / 10.0);

        // Trapezoidal integration of noise power
        let power_area = (l1 + l2) / 2.0 * (f2 - f1);

        // Convert phase noise area (rad^2) to jitter^2
        // Jitter^2 = 2 * Area / (2*pi*f_c)^2  <-- Approximation for single sideband
        // Correct relation: RMS Jitter = sqrt(2 * Integral(L(f))) / (2*pi*f_vco)
        jitter_sq += power_area;
    }

    let rms_jitter_seconds = (2.0 * jitter_sq).sqrt() / (2.0 * PI * vco_freq_hz);

    PhaseNoiseProfile {
        offsets_hz: offsets,
        total_dbc_hz: total_noise,
        integrated_jitter_fs: rms_jitter_seconds * 1e15,
        components: components_list,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pll::types::BodePlot;

    #[test]
    fn test_phase_noise_calculation() {
        // Create a dummy Bode plot
        let frequencies: Vec<f64> = (3..8).map(|i| 10f64.powi(i)).collect(); // 1k to 100M
        let magnitude_db = vec![40.0, 20.0, 0.0, -20.0, -40.0]; // Simple roll-off
        let phase_deg = vec![-90.0; 5];

        let bode = BodePlot {
            frequencies_hz: frequencies,
            magnitude_db,
            phase_deg,
        };

        let profile = calculate_phase_noise(
            &bode, 100.0, // N
            10e6,  // Ref
            1e9,   // VCO
        );

        assert_eq!(profile.offsets_hz.len(), 5);
        assert!(profile.integrated_jitter_fs > 0.0);
        assert!(profile.components[0].total_dbc_hz < 0.0);
    }
}
