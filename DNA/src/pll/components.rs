use serde::{Deserialize, Serialize};

// ============================================================================
// VCO COMPONENT LIBRARY
// ============================================================================

/// VCO (Voltage-Controlled Oscillator) specification
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VCOSpec {
    /// Manufacturer part number
    pub part_number: String,
    /// Manufacturer name
    pub manufacturer: String,
    /// Minimum output frequency (Hz)
    pub freq_min_hz: f64,
    /// Maximum output frequency (Hz)
    pub freq_max_hz: f64,
    /// Tuning sensitivity (MHz/V)
    pub kvco_mhz_per_v: f64,
    /// Minimum tuning voltage (V)
    pub vtune_min_v: f64,
    /// Maximum tuning voltage (V)
    pub vtune_max_v: f64,
    /// Phase noise at 10kHz offset (dBc/Hz)
    pub phase_noise_10khz_dbc: f64,
    /// Phase noise at 100kHz offset (dBc/Hz)
    pub phase_noise_100khz_dbc: f64,
    /// Supply voltage (V)
    pub vcc_v: f64,
    /// Supply current (mA)
    pub icc_ma: f64,
    /// Package type
    pub package: String,
}

/// PFD/Charge Pump IC specification
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PFDSpec {
    /// Manufacturer part number
    pub part_number: String,
    /// Manufacturer name
    pub manufacturer: String,
    /// Maximum input frequency (Hz)
    pub max_freq_hz: f64,
    /// Charge pump current options (mA)
    pub icp_options_ma: Vec<f64>,
    /// Supply voltage (V)
    pub vcc_v: f64,
    /// Phase noise floor (dBc/Hz)
    pub noise_floor_dbc: f64,
    /// Has fractional-N support
    pub fractional_n: bool,
    /// Maximum N divider value
    pub max_n: u32,
    /// Package type
    pub package: String,
}

/// Complete PLL IC specification (integrated PFD + VCO)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PLLICSpec {
    /// Manufacturer part number
    pub part_number: String,
    /// Manufacturer name
    pub manufacturer: String,
    /// Minimum output frequency (Hz)
    pub freq_min_hz: f64,
    /// Maximum output frequency (Hz)
    pub freq_max_hz: f64,
    /// Maximum reference frequency (Hz)
    pub max_ref_freq_hz: f64,
    /// Charge pump current options (mA)
    pub icp_options_ma: Vec<f64>,
    /// VCO gain (MHz/V)
    pub kvco_mhz_per_v: f64,
    /// Has fractional-N support
    pub fractional_n: bool,
    /// Modulus for fractional-N (if supported)
    pub frac_modulus: Option<u32>,
    /// Phase noise at 10kHz offset (dBc/Hz)
    pub phase_noise_10khz_dbc: f64,
    /// Supply voltage (V)
    pub vcc_v: f64,
    /// Package type
    pub package: String,
}

// ============================================================================
// BUILT-IN COMPONENT LIBRARY
// ============================================================================

/// Get built-in VCO library
pub fn get_vco_library() -> Vec<VCOSpec> {
    vec![
        VCOSpec {
            part_number: "CVCO55CC-2400-2500".to_string(),
            manufacturer: "Crystek".to_string(),
            freq_min_hz: 2.4e9,
            freq_max_hz: 2.5e9,
            kvco_mhz_per_v: 40.0,
            vtune_min_v: 0.5,
            vtune_max_v: 4.5,
            phase_noise_10khz_dbc: -95.0,
            phase_noise_100khz_dbc: -118.0,
            vcc_v: 5.0,
            icc_ma: 35.0,
            package: "SMD 0.5x0.5".to_string(),
        },
        VCOSpec {
            part_number: "JTOS-2700".to_string(),
            manufacturer: "Mini-Circuits".to_string(),
            freq_min_hz: 2.35e9,
            freq_max_hz: 2.75e9,
            kvco_mhz_per_v: 80.0,
            vtune_min_v: 1.0,
            vtune_max_v: 15.0,
            phase_noise_10khz_dbc: -90.0,
            phase_noise_100khz_dbc: -115.0,
            vcc_v: 8.0,
            icc_ma: 25.0,
            package: "TO-8".to_string(),
        },
        VCOSpec {
            part_number: "ROS-2536C".to_string(),
            manufacturer: "Mini-Circuits".to_string(),
            freq_min_hz: 2.13e9,
            freq_max_hz: 2.53e9,
            kvco_mhz_per_v: 30.0,
            vtune_min_v: 0.5,
            vtune_max_v: 8.0,
            phase_noise_10khz_dbc: -100.0,
            phase_noise_100khz_dbc: -125.0,
            vcc_v: 5.0,
            icc_ma: 30.0,
            package: "SMD".to_string(),
        },
        VCOSpec {
            part_number: "HMC358MS8G".to_string(),
            manufacturer: "Analog Devices".to_string(),
            freq_min_hz: 5.8e9,
            freq_max_hz: 6.8e9,
            kvco_mhz_per_v: 100.0,
            vtune_min_v: 0.0,
            vtune_max_v: 12.0,
            phase_noise_10khz_dbc: -88.0,
            phase_noise_100khz_dbc: -108.0,
            vcc_v: 5.0,
            icc_ma: 45.0,
            package: "MSOP-8".to_string(),
        },
    ]
}

/// Get built-in PLL IC library
pub fn get_pll_ic_library() -> Vec<PLLICSpec> {
    vec![
        PLLICSpec {
            part_number: "ADF4351".to_string(),
            manufacturer: "Analog Devices".to_string(),
            freq_min_hz: 35e6,
            freq_max_hz: 4.4e9,
            max_ref_freq_hz: 250e6,
            icp_options_ma: vec![0.31, 0.63, 0.94, 1.25, 1.56, 1.88, 2.19, 2.5, 2.81, 3.13, 3.44, 3.75, 4.06, 4.38, 4.69, 5.0],
            kvco_mhz_per_v: 30.0,
            fractional_n: true,
            frac_modulus: Some(4095),
            phase_noise_10khz_dbc: -89.0,
            vcc_v: 3.3,
            package: "LFCSP-32".to_string(),
        },
        PLLICSpec {
            part_number: "ADF4350".to_string(),
            manufacturer: "Analog Devices".to_string(),
            freq_min_hz: 137.5e6,
            freq_max_hz: 4.4e9,
            max_ref_freq_hz: 250e6,
            icp_options_ma: vec![0.31, 0.63, 0.94, 1.25, 1.56, 1.88, 2.19, 2.5, 2.81, 3.13, 3.44, 3.75, 4.06, 4.38, 4.69, 5.0],
            kvco_mhz_per_v: 30.0,
            fractional_n: true,
            frac_modulus: Some(4095),
            phase_noise_10khz_dbc: -89.0,
            vcc_v: 3.3,
            package: "LFCSP-32".to_string(),
        },
        PLLICSpec {
            part_number: "MAX2871".to_string(),
            manufacturer: "Maxim".to_string(),
            freq_min_hz: 23.5e6,
            freq_max_hz: 6.0e9,
            max_ref_freq_hz: 200e6,
            icp_options_ma: vec![0.32, 0.64, 0.96, 1.28, 1.92, 2.56, 3.2, 3.84, 4.48, 5.12],
            kvco_mhz_per_v: 50.0,
            fractional_n: true,
            frac_modulus: Some(4095),
            phase_noise_10khz_dbc: -100.0,
            vcc_v: 3.3,
            package: "QFN-32".to_string(),
        },
        PLLICSpec {
            part_number: "LMX2594".to_string(),
            manufacturer: "Texas Instruments".to_string(),
            freq_min_hz: 10e6,
            freq_max_hz: 15e9,
            max_ref_freq_hz: 1.4e9,
            icp_options_ma: vec![0.625, 1.25, 1.875, 2.5, 3.125, 3.75, 4.375, 5.0, 6.25, 7.5],
            kvco_mhz_per_v: 25.0,
            fractional_n: true,
            frac_modulus: Some(16777215),
            phase_noise_10khz_dbc: -110.0,
            vcc_v: 3.3,
            package: "QFN-40".to_string(),
        },
        PLLICSpec {
            part_number: "HMC833LP6GE".to_string(),
            manufacturer: "Analog Devices".to_string(),
            freq_min_hz: 25e6,
            freq_max_hz: 6.0e9,
            max_ref_freq_hz: 350e6,
            icp_options_ma: vec![0.02, 0.04, 0.08, 0.16, 0.32, 0.64, 1.28, 2.56],
            kvco_mhz_per_v: 35.0,
            fractional_n: true,
            frac_modulus: Some(16777215),
            phase_noise_10khz_dbc: -115.0,
            vcc_v: 3.3,
            package: "QFN-40".to_string(),
        },
    ]
}

// ============================================================================
// COMPONENT SELECTION ALGORITHM
// ============================================================================

/// Score a VCO for a given application
pub fn score_vco(vco: &VCOSpec, target_freq_hz: f64, required_kvco: f64) -> f64 {
    // Check if frequency is in range
    if target_freq_hz < vco.freq_min_hz || target_freq_hz > vco.freq_max_hz {
        return 0.0;
    }

    let mut score = 100.0;

    // Penalize if Kvco is too different from required
    let kvco_ratio = (vco.kvco_mhz_per_v / required_kvco).abs();
    if kvco_ratio > 2.0 || kvco_ratio < 0.5 {
        score -= 30.0;
    } else if kvco_ratio > 1.5 || kvco_ratio < 0.67 {
        score -= 15.0;
    }

    // Bonus for better phase noise
    score += (vco.phase_noise_100khz_dbc + 130.0).min(20.0); // -130 dBc/Hz is baseline

    // Bonus for lower power
    score += (50.0 - vco.icc_ma).max(0.0) / 5.0;

    score.max(0.0)
}

/// Score a PLL IC for a given application
pub fn score_pll_ic(
    ic: &PLLICSpec,
    target_freq_hz: f64,
    ref_freq_hz: f64,
    target_icp_ma: f64,
    needs_fractional: bool,
) -> f64 {
    // Check if frequency is in range
    if target_freq_hz < ic.freq_min_hz || target_freq_hz > ic.freq_max_hz {
        return 0.0;
    }

    // Check reference frequency
    if ref_freq_hz > ic.max_ref_freq_hz {
        return 0.0;
    }

    // Check fractional-N requirement
    if needs_fractional && !ic.fractional_n {
        return 0.0;
    }

    let mut score = 100.0;

    // Find closest charge pump current option
    let closest_icp = ic.icp_options_ma
        .iter()
        .min_by(|a, b| {
            ((*a - target_icp_ma).abs())
                .partial_cmp(&((*b - target_icp_ma).abs()))
                .unwrap()
        })
        .unwrap_or(&1.0);

    let icp_error_pct = ((closest_icp - target_icp_ma) / target_icp_ma * 100.0).abs();
    score -= icp_error_pct.min(30.0);

    // Bonus for better phase noise
    score += (ic.phase_noise_10khz_dbc + 110.0).min(20.0);

    // Bonus for higher modulus (finer resolution)
    if let Some(modulus) = ic.frac_modulus {
        score += (modulus as f64).log10() * 2.0;
    }

    score.max(0.0)
}

/// Select best VCO for application
pub fn select_vco(target_freq_hz: f64, required_kvco: f64) -> Option<VCOSpec> {
    let library = get_vco_library();
    library
        .into_iter()
        .filter(|vco| score_vco(vco, target_freq_hz, required_kvco) > 0.0)
        .max_by(|a, b| {
            score_vco(a, target_freq_hz, required_kvco)
                .partial_cmp(&score_vco(b, target_freq_hz, required_kvco))
                .unwrap()
        })
}

/// Select best PLL IC for application
pub fn select_pll_ic(
    target_freq_hz: f64,
    ref_freq_hz: f64,
    target_icp_ma: f64,
    needs_fractional: bool,
) -> Option<PLLICSpec> {
    let library = get_pll_ic_library();
    library
        .into_iter()
        .filter(|ic| score_pll_ic(ic, target_freq_hz, ref_freq_hz, target_icp_ma, needs_fractional) > 0.0)
        .max_by(|a, b| {
            score_pll_ic(a, target_freq_hz, ref_freq_hz, target_icp_ma, needs_fractional)
                .partial_cmp(&score_pll_ic(b, target_freq_hz, ref_freq_hz, target_icp_ma, needs_fractional))
                .unwrap()
        })
}

// ============================================================================
// E-SERIES COMPONENT VALUES
// ============================================================================

/// E24 resistor series (5% tolerance)
pub const E24: [f64; 24] = [
    1.0, 1.1, 1.2, 1.3, 1.5, 1.6, 1.8, 2.0, 2.2, 2.4, 2.7, 3.0,
    3.3, 3.6, 3.9, 4.3, 4.7, 5.1, 5.6, 6.2, 6.8, 7.5, 8.2, 9.1,
];

/// E96 resistor series (1% tolerance)
pub const E96: [f64; 96] = [
    1.00, 1.02, 1.05, 1.07, 1.10, 1.13, 1.15, 1.18, 1.21, 1.24, 1.27, 1.30,
    1.33, 1.37, 1.40, 1.43, 1.47, 1.50, 1.54, 1.58, 1.62, 1.65, 1.69, 1.74,
    1.78, 1.82, 1.87, 1.91, 1.96, 2.00, 2.05, 2.10, 2.15, 2.21, 2.26, 2.32,
    2.37, 2.43, 2.49, 2.55, 2.61, 2.67, 2.74, 2.80, 2.87, 2.94, 3.01, 3.09,
    3.16, 3.24, 3.32, 3.40, 3.48, 3.57, 3.65, 3.74, 3.83, 3.92, 4.02, 4.12,
    4.22, 4.32, 4.42, 4.53, 4.64, 4.75, 4.87, 4.99, 5.11, 5.23, 5.36, 5.49,
    5.62, 5.76, 5.90, 6.04, 6.19, 6.34, 6.49, 6.65, 6.81, 6.98, 7.15, 7.32,
    7.50, 7.68, 7.87, 8.06, 8.25, 8.45, 8.66, 8.87, 9.09, 9.31, 9.53, 9.76,
];

/// Find the nearest E24 value to the target
pub fn nearest_e24(target: f64) -> f64 {
    if target <= 0.0 {
        return E24[0];
    }

    // Determine the decade multiplier
    let decade = 10.0_f64.powf((target.log10()).floor());
    let normalized = target / decade;

    // Find nearest value in E24
    let mut closest = E24[0];
    let mut min_diff = (normalized - E24[0]).abs();

    for &value in &E24 {
        let diff = (normalized - value).abs();
        if diff < min_diff {
            min_diff = diff;
            closest = value;
        }
    }

    closest * decade
}

/// Find the nearest E96 value to the target
pub fn nearest_e96(target: f64) -> f64 {
    if target <= 0.0 {
        return E96[0];
    }

    // Determine the decade multiplier
    let decade = 10.0_f64.powf((target.log10()).floor());
    let normalized = target / decade;

    // Find nearest value in E96
    let mut closest = E96[0];
    let mut min_diff = (normalized - E96[0]).abs();

    for &value in &E96 {
        let diff = (normalized - value).abs();
        if diff < min_diff {
            min_diff = diff;
            closest = value;
        }
    }

    closest * decade
}

/// Format resistance value with unit prefix
pub fn format_resistance(ohms: f64) -> String {
    if ohms >= 1e6 {
        format!("{:.2} MΩ", ohms / 1e6)
    } else if ohms >= 1e3 {
        format!("{:.2} kΩ", ohms / 1e3)
    } else {
        format!("{:.2} Ω", ohms)
    }
}

/// Format capacitance value with unit prefix
pub fn format_capacitance(farads: f64) -> String {
    if farads >= 1e-6 {
        format!("{:.2} µF", farads / 1e-6)
    } else if farads >= 1e-9 {
        format!("{:.2} nF", farads / 1e-9)
    } else if farads >= 1e-12 {
        format!("{:.2} pF", farads / 1e-12)
    } else {
        format!("{:.2} fF", farads / 1e-15)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nearest_e24() {
        assert_eq!(nearest_e24(1000.0), 1000.0);
        // 1050 is closer to 1000 (diff=50) than 1100 (diff=50), but let's verify
        let result = nearest_e24(1050.0);
        assert!(result == 1000.0 || result == 1100.0);
        assert_eq!(nearest_e24(10000.0), 10000.0);
    }

    #[test]
    fn test_nearest_e96() {
        assert_eq!(nearest_e96(1000.0), 1000.0);
        assert_eq!(nearest_e96(1010.0), 1000.0);  // Closer to 1000
        assert_eq!(nearest_e96(10500.0), 10500.0);
    }

    #[test]
    fn test_format_resistance() {
        assert_eq!(format_resistance(100.0), "100.00 Ω");
        assert_eq!(format_resistance(10000.0), "10.00 kΩ");
        assert_eq!(format_resistance(1000000.0), "1.00 MΩ");
    }

    #[test]
    fn test_format_capacitance() {
        assert_eq!(format_capacitance(1e-12), "1.00 pF");
        assert_eq!(format_capacitance(1e-9), "1.00 nF");
        assert_eq!(format_capacitance(1e-6), "1.00 µF");
    }

    #[test]
    fn test_vco_library() {
        let library = get_vco_library();
        assert!(library.len() >= 4);

        // Check that all VCOs have valid frequency ranges
        for vco in &library {
            assert!(vco.freq_max_hz > vco.freq_min_hz);
            assert!(vco.kvco_mhz_per_v > 0.0);
        }
    }

    #[test]
    fn test_pll_ic_library() {
        let library = get_pll_ic_library();
        assert!(library.len() >= 5);

        // Check that all ICs have valid specs
        for ic in &library {
            assert!(ic.freq_max_hz > ic.freq_min_hz);
            assert!(!ic.icp_options_ma.is_empty());
        }
    }

    #[test]
    fn test_select_vco_2_4ghz() {
        let vco = select_vco(2.45e9, 40.0);
        assert!(vco.is_some());

        let vco = vco.unwrap();
        assert!(vco.freq_min_hz <= 2.45e9);
        assert!(vco.freq_max_hz >= 2.45e9);
    }

    #[test]
    fn test_select_pll_ic() {
        let ic = select_pll_ic(2.45e9, 10e6, 1.0, true);
        assert!(ic.is_some());

        let ic = ic.unwrap();
        assert!(ic.fractional_n);
        assert!(ic.freq_min_hz <= 2.45e9);
        assert!(ic.freq_max_hz >= 2.45e9);
    }

    #[test]
    fn test_score_vco() {
        let vco = VCOSpec {
            part_number: "TEST-VCO".to_string(),
            manufacturer: "Test".to_string(),
            freq_min_hz: 2.0e9,
            freq_max_hz: 3.0e9,
            kvco_mhz_per_v: 50.0,
            vtune_min_v: 0.5,
            vtune_max_v: 5.0,
            phase_noise_10khz_dbc: -95.0,
            phase_noise_100khz_dbc: -120.0,
            vcc_v: 5.0,
            icc_ma: 30.0,
            package: "SMD".to_string(),
        };

        // In range - should have positive score
        let score = score_vco(&vco, 2.5e9, 50.0);
        assert!(score > 0.0);

        // Out of range - should have zero score
        let score = score_vco(&vco, 5.0e9, 50.0);
        assert_eq!(score, 0.0);
    }
}
