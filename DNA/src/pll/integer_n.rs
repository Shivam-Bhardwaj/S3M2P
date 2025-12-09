use super::types::DividerConfig;

/// Calculate optimal integer-N dividers for PLL
///
/// Given reference and output frequencies, finds R and N dividers that:
/// - Maximize PFD frequency (f_pfd = f_ref / R)
/// - Satisfy f_out = f_ref * (N / R)
/// - Keep dividers within reasonable ranges
pub fn calculate_dividers(ref_freq_hz: f64, output_freq_hz: f64) -> (u32, u32, f64) {
    // Start with R = 1 to maximize PFD frequency
    // Then try higher R values if N becomes too large

    let max_n = 65535; // Typical maximum for PLL ICs
    let min_pfd = 1e6; // Minimum PFD frequency (1 MHz)

    for r in 1..=16 {
        let r_f64 = r as f64;
        let pfd_freq = ref_freq_hz / r_f64;

        // Check minimum PFD frequency
        if pfd_freq < min_pfd {
            break;
        }

        // Calculate required N
        let n_exact = output_freq_hz / pfd_freq;
        let n = n_exact.round() as u32;

        // Check if N is within range
        if n > 0 && n <= max_n {
            let pfd_freq_hz = ref_freq_hz / r_f64;
            return (r, n, pfd_freq_hz);
        }
    }

    // Fallback: use larger R if needed
    let r = (ref_freq_hz / min_pfd).ceil() as u32;
    let pfd_freq = ref_freq_hz / r as f64;
    let n = (output_freq_hz / pfd_freq).round() as u32;

    (r, n, pfd_freq)
}

/// Create integer-N divider configuration
pub fn create_integer_n_config(n: u32) -> DividerConfig {
    // Determine if prescaler is needed (for high N values)
    if n > 1024 {
        // Use prescaler (typically P=8 or P=16)
        let prescaler = if n > 8192 { 16 } else { 8 };
        DividerConfig::IntegerN {
            n,
            prescaler: Some(prescaler),
        }
    } else {
        DividerConfig::IntegerN { n, prescaler: None }
    }
}

/// Calculate actual output frequency achieved
pub fn calculate_output_freq(ref_freq_hz: f64, r: u32, n: u32) -> f64 {
    let pfd_freq = ref_freq_hz / r as f64;
    pfd_freq * n as f64
}

/// Calculate frequency error
pub fn calculate_freq_error(target_hz: f64, actual_hz: f64) -> f64 {
    (actual_hz - target_hz) / target_hz
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_dividers() {
        // Test case: 10 MHz ref -> 2.4 GHz output
        let (r, n, pfd_freq) = calculate_dividers(10e6, 2.4e9);
        let actual_output = calculate_output_freq(10e6, r, n);
        let error = calculate_freq_error(2.4e9, actual_output);

        assert_eq!(r, 1); // Should maximize PFD frequency
        assert_eq!(n, 240);
        assert!((pfd_freq - 10e6).abs() < 1.0);
        assert!(error.abs() < 1e-6); // Less than 1 ppm error
    }

    #[test]
    fn test_low_output_frequency() {
        // Test case: 10 MHz ref -> 100 MHz output
        let (r, n, _pfd_freq) = calculate_dividers(10e6, 100e6);
        let actual_output = calculate_output_freq(10e6, r, n);

        assert_eq!(r, 1);
        assert_eq!(n, 10);
        assert!((actual_output - 100e6).abs() < 1.0);
    }

    #[test]
    fn test_prescaler_selection() {
        let config_small = create_integer_n_config(100);
        match config_small {
            DividerConfig::IntegerN { n, prescaler } => {
                assert_eq!(n, 100);
                assert_eq!(prescaler, None);
            }
            _ => panic!("Expected IntegerN config"),
        }

        let config_large = create_integer_n_config(2000);
        match config_large {
            DividerConfig::IntegerN { n, prescaler } => {
                assert_eq!(n, 2000);
                assert_eq!(prescaler, Some(8));
            }
            _ => panic!("Expected IntegerN config"),
        }
    }
}
