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
}
