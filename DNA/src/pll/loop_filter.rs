use super::components::{format_capacitance, format_resistance, nearest_e96};
use super::types::{FilterComponent, LoopFilterDesign, LoopFilterTopology};
use std::f64::consts::PI;

/// Design a passive 2nd order loop filter
///
/// Parameters:
/// - k_phi: Charge pump current (A/rad)
/// - k_vco: VCO gain (Hz/V)
/// - n: Feedback divider
/// - omega_c: Target crossover frequency (rad/s)
/// - phase_margin_deg: Target phase margin (degrees)
///
/// Returns: (C1, R1, C2) in SI units
pub fn design_passive_second_order(
    k_phi: f64,
    k_vco: f64,
    n: f64,
    omega_c: f64,
    phase_margin_deg: f64,
) -> (f64, f64, f64) {
    // Convert phase margin to radians
    let pm_rad = phase_margin_deg * PI / 180.0;

    // Calculate T1 and T2 time constants
    let t1 = (1.0 / pm_rad.cos() - pm_rad.tan()) / omega_c;
    let t2 = 1.0 / (omega_c * omega_c * t1);

    // Calculate C1 from loop gain requirement
    let c1 = (k_phi * k_vco) / (n * omega_c * omega_c);

    // Calculate R1 and C2
    let r1 = t1 / c1;
    let c2 = t2 / r1;

    (c1, r1, c2)
}

/// Design a passive 3rd order loop filter (adds extra pole for spur attenuation)
///
/// The 3rd pole is placed at 5-10x the crossover frequency
pub fn design_passive_third_order(
    k_phi: f64,
    k_vco: f64,
    n: f64,
    omega_c: f64,
    phase_margin_deg: f64,
) -> (f64, f64, f64, f64, f64) {
    // Design 2nd order filter first
    let (c1, r1, c2) = design_passive_second_order(k_phi, k_vco, n, omega_c, phase_margin_deg);

    // Place 3rd pole at 7x crossover frequency
    let omega_p3 = 7.0 * omega_c;

    // C3 is typically 1/5 of C2
    let c3 = c2 / 5.0;

    // R2 sets the 3rd pole location
    let r2 = 1.0 / (omega_p3 * c3);

    (c1, r1, c2, r2, c3)
}

/// Create a complete loop filter design with E96 values
pub fn create_loop_filter_design(
    c1: f64,
    r1: f64,
    c2: f64,
    c3: Option<f64>,
    r2: Option<f64>,
) -> LoopFilterDesign {
    let mut components = Vec::new();

    // C1 (main integrating capacitor) - use exact value or nearest
    let c1_actual = if c1 < 1e-9 {
        c1 // Small caps, use ideal
    } else {
        nearest_e96(c1 / 1e-9) * 1e-9 // Normalize to nF
    };

    components.push(FilterComponent {
        designator: "C1".to_string(),
        value: c1,
        actual_value: c1_actual,
        unit: format_capacitance(c1_actual),
        tolerance_pct: 10.0,
    });

    // R1 (zero resistor) - use E96 series
    let r1_actual = nearest_e96(r1);
    components.push(FilterComponent {
        designator: "R1".to_string(),
        value: r1,
        actual_value: r1_actual,
        unit: format_resistance(r1_actual),
        tolerance_pct: 1.0,
    });

    // C2 (pole capacitor) - use nearest value
    let c2_actual = if c2 < 1e-9 {
        c2
    } else {
        nearest_e96(c2 / 1e-9) * 1e-9
    };

    components.push(FilterComponent {
        designator: "C2".to_string(),
        value: c2,
        actual_value: c2_actual,
        unit: format_capacitance(c2_actual),
        tolerance_pct: 10.0,
    });

    let (c3_val, r2_val, topology) = if let (Some(c3), Some(r2)) = (c3, r2) {
        let c3_actual = if c3 < 1e-9 {
            c3
        } else {
            nearest_e96(c3 / 1e-9) * 1e-9
        };
        let r2_actual = nearest_e96(r2);

        components.push(FilterComponent {
            designator: "C3".to_string(),
            value: c3,
            actual_value: c3_actual,
            unit: format_capacitance(c3_actual),
            tolerance_pct: 10.0,
        });

        components.push(FilterComponent {
            designator: "R2".to_string(),
            value: r2,
            actual_value: r2_actual,
            unit: format_resistance(r2_actual),
            tolerance_pct: 1.0,
        });

        (
            Some(c3_actual),
            Some(r2_actual),
            LoopFilterTopology::PassiveThirdOrder,
        )
    } else {
        (None, None, LoopFilterTopology::PassiveSecondOrder)
    };

    LoopFilterDesign {
        topology,
        components,
        c1_pf: c1_actual * 1e12,
        c2_pf: c2_actual * 1e12,
        r1_ohms: r1_actual,
        c3_pf: c3_val.map(|v| v * 1e12),
        r2_ohms: r2_val,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_second_order_design() {
        // Typical values for 2.4 GHz PLL
        let k_phi = 1e-3; // 1 mA charge pump
        let k_vco = 10e6; // 10 MHz/V
        let n = 240.0;
        let f_c = 100e3; // 100 kHz bandwidth
        let omega_c = 2.0 * PI * f_c;
        let pm = 45.0;

        let (c1, r1, c2) = design_passive_second_order(k_phi, k_vco, n, omega_c, pm);

        // Verify values are reasonable
        assert!(c1 > 0.0);
        assert!(r1 > 0.0);
        assert!(c2 > 0.0);

        // Verify time constants exist
        let _t1 = r1 * c1;
        let _t2 = r1 * c2;
    }

    #[test]
    fn test_third_order_design() {
        let k_phi = 1e-3;
        let k_vco = 10e6;
        let n = 240.0;
        let f_c = 100e3;
        let omega_c = 2.0 * PI * f_c;
        let pm = 45.0;

        let (_c1, _r1, c2, r2, c3) = design_passive_third_order(k_phi, k_vco, n, omega_c, pm);

        // Verify 3rd order components
        assert!(c3 > 0.0);
        assert!(r2 > 0.0);
        assert!(c3 < c2); // C3 typically smaller than C2
    }

    #[test]
    fn test_create_loop_filter_design() {
        let design = create_loop_filter_design(1e-9, 10e3, 100e-12, None, None);

        assert_eq!(design.topology, LoopFilterTopology::PassiveSecondOrder);
        assert_eq!(design.components.len(), 3);
        assert_eq!(design.components[0].designator, "C1");
        assert_eq!(design.components[1].designator, "R1");
        assert_eq!(design.components[2].designator, "C2");
    }
}
