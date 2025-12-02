use crate::pll::types::*;

/// Calculate fractional-N divider parameters
///
/// Fractional-N PLLs allow non-integer division ratios, enabling finer
/// frequency resolution than Integer-N PLLs.
///
/// The divider alternates between N and N+1, controlled by a sigma-delta
/// modulator to average out to the desired fractional value.
///
/// ## Example
/// ```
/// use dna::pll::fractional_n::*;
///
/// let (n_int, n_frac, modulus) = calculate_fractional_divider(10e6, 2.4453e9, 3);
/// assert_eq!(n_int, 244);  // Integer part
/// // n_frac/modulus ≈ 0.53  // Fractional part
/// ```
pub fn calculate_fractional_divider(
    pfd_freq_hz: f64,
    output_freq_hz: f64,
    modulator_order: u32,
) -> (u32, u32, u32) {
    // Calculate exact division ratio
    let n_exact = output_freq_hz / pfd_freq_hz;

    // Integer part
    let n_int = n_exact.floor() as u32;

    // Fractional part
    let n_frac_float = n_exact - n_int as f64;

    // Modulus depends on modulator order
    // Order 1: 2^16 = 65536
    // Order 2: 2^20 = 1048576
    // Order 3: 2^24 = 16777216
    let modulus = match modulator_order {
        1 => 1u32 << 16,  // 65536
        2 => 1u32 << 20,  // 1048576
        3 => 1u32 << 24,  // 16777216
        _ => 1u32 << 24,  // Default to order 3
    };

    // Convert fractional part to integer numerator
    let n_frac = (n_frac_float * modulus as f64).round() as u32;

    (n_int, n_frac, modulus)
}

/// Create fractional-N divider configuration
pub fn create_fractional_n_config(
    n_int: u32,
    n_frac: u32,
    modulus: u32,
    modulator_order: u32,
) -> DividerConfig {
    DividerConfig::FractionalN {
        n_int,
        n_frac,
        modulus,
        modulator_order,
    }
}

/// Estimate quantization noise from sigma-delta modulator
///
/// Quantization noise is the primary disadvantage of fractional-N PLLs.
/// Higher order modulators provide better noise shaping.
///
/// ## Noise Characteristics
/// - **Order 1**: 10 dB/decade rolloff, simple but noisy
/// - **Order 2**: 20 dB/decade rolloff, moderate noise
/// - **Order 3**: 30 dB/decade rolloff, best noise performance
///
/// Returns noise spectral density in dBc/Hz at given offset frequency
pub fn estimate_quantization_noise(
    modulator_order: u32,
    pfd_freq_hz: f64,
    offset_freq_hz: f64,
) -> f64 {
    // Quantization noise floor (varies by modulator order)
    let noise_floor_dbchz = match modulator_order {
        1 => -115.0,
        2 => -125.0,
        3 => -135.0,
        _ => -135.0,
    };

    // High-pass noise shaping: noise increases with frequency
    // Slope: modulator_order * 20 dB/decade
    let slope = modulator_order as f64 * 20.0;

    // Calculate noise at offset frequency
    // Normalized to PFD frequency (noise corner)
    let freq_ratio = offset_freq_hz / pfd_freq_hz;

    if freq_ratio < 1e-6 {
        // DC: minimum noise
        noise_floor_dbchz
    } else if freq_ratio < 0.5 {
        // Within PFD/2: noise increases with slope
        noise_floor_dbchz + slope * freq_ratio.log10()
    } else {
        // Beyond PFD/2: aliasing dominates
        noise_floor_dbchz + slope * 0.5f64.log10() + 10.0
    }
}

/// Design fractional-N loop filter with increased bandwidth
///
/// Fractional-N PLLs typically use wider loop bandwidth to suppress
/// quantization noise from the sigma-delta modulator.
pub fn adjust_bandwidth_for_fractional(
    integer_n_bandwidth_hz: f64,
    modulator_order: u32,
) -> f64 {
    // Fractional-N typically needs 2-5x wider bandwidth
    let bandwidth_multiplier = match modulator_order {
        1 => 5.0,  // Order 1: very noisy, need wide bandwidth
        2 => 3.0,  // Order 2: moderate noise
        3 => 2.0,  // Order 3: good noise shaping, less bandwidth needed
        _ => 2.0,
    };

    integer_n_bandwidth_hz * bandwidth_multiplier
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_fractional_divider() {
        // Test case: 2.4453 GHz output from 10 MHz PFD
        let (n_int, n_frac, modulus) = calculate_fractional_divider(10e6, 2.4453e9, 3);

        assert_eq!(n_int, 244);
        assert_eq!(modulus, 16777216); // 2^24

        // Check that n_frac gives approximately 0.53
        let frac_part = n_frac as f64 / modulus as f64;
        assert!((frac_part - 0.53).abs() < 0.01);

        // Verify actual output frequency
        let actual_freq = (n_int as f64 + frac_part) * 10e6;
        assert!((actual_freq - 2.4453e9).abs() < 1e3); // Within 1 kHz
    }

    #[test]
    fn test_create_fractional_n_config() {
        let config = create_fractional_n_config(244, 8888888, 16777216, 3);

        match config {
            DividerConfig::FractionalN {
                n_int,
                n_frac,
                modulus,
                modulator_order,
            } => {
                assert_eq!(n_int, 244);
                assert_eq!(n_frac, 8888888);
                assert_eq!(modulus, 16777216);
                assert_eq!(modulator_order, 3);
            }
            _ => panic!("Expected FractionalN config"),
        }
    }

    #[test]
    fn test_quantization_noise() {
        let pfd_freq = 10e6;

        // At low offset (100 Hz), noise should be very low
        let noise_low = estimate_quantization_noise(3, pfd_freq, 100.0);
        assert!(noise_low < -100.0); // Better than -100 dBc/Hz

        // At high offset (1 MHz), noise increases
        let noise_high = estimate_quantization_noise(3, pfd_freq, 1e6);
        assert!(noise_high > noise_low); // Higher noise at higher offset

        // Order 3 should be better than Order 1
        let noise_order1 = estimate_quantization_noise(1, pfd_freq, 1e6);
        let noise_order3 = estimate_quantization_noise(3, pfd_freq, 1e6);
        assert!(noise_order3 < noise_order1);
    }

    #[test]
    fn test_bandwidth_adjustment() {
        let base_bw = 100e3; // 100 kHz

        let bw_order1 = adjust_bandwidth_for_fractional(base_bw, 1);
        let bw_order2 = adjust_bandwidth_for_fractional(base_bw, 2);
        let bw_order3 = adjust_bandwidth_for_fractional(base_bw, 3);

        // Higher order = less bandwidth increase needed
        assert!(bw_order1 > bw_order2);
        assert!(bw_order2 > bw_order3);
        assert!(bw_order3 >= base_bw * 2.0);
    }
}
