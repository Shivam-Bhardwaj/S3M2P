pub mod circuit;
pub mod components;
pub mod integer_n;
pub mod loop_filter;
pub mod stability;
pub mod types;

use std::f64::consts::PI;

pub use types::*;

/// Main entry point: Design a PLL from requirements
pub fn design_pll(requirements: &PLLRequirements) -> Result<PLLDesign, String> {
    // Validate requirements
    let validation = requirements.validate();
    if !validation.is_valid {
        return Err(format!("Invalid requirements: {:?}", validation.errors));
    }

    // Use center frequency for design
    let output_freq_hz = (requirements.output_freq_min_hz + requirements.output_freq_max_hz) / 2.0;

    // Calculate dividers
    let (r, n, pfd_freq_hz) = integer_n::calculate_dividers(requirements.ref_freq_hz, output_freq_hz);

    // Create divider configuration
    let divider_n = integer_n::create_integer_n_config(n);

    // Typical PLL IC parameters
    let charge_pump_current_ua = 1000.0;  // 1 mA
    let k_phi = charge_pump_current_ua * 1e-6;  // Convert to A

    // Typical VCO gain (will be replaced with component selection later)
    let vco_gain_mhz_per_v = 10.0;
    let k_vco = vco_gain_mhz_per_v * 1e6 * 2.0 * PI;  // Convert to rad/s/V

    // Design loop filter
    let omega_c = 2.0 * PI * requirements.loop_bandwidth_hz;
    let (c1, r1, c2) = loop_filter::design_passive_second_order(
        k_phi,
        k_vco,
        n as f64,
        omega_c,
        requirements.phase_margin_deg,
    );

    let loop_filter_design = loop_filter::create_loop_filter_design(c1, r1, c2, None, None);

    // Generate Bode plot
    let bode_plot = stability::generate_bode_plot(
        k_phi,
        k_vco,
        n as f64,
        loop_filter_design.r1_ohms,
        loop_filter_design.c1_pf * 1e-12,
        loop_filter_design.c2_pf * 1e-12,
        1e3,   // 1 kHz start
        10.0 * requirements.ref_freq_hz,  // 10x ref freq stop
        50,    // 50 points per decade
    );

    // Analyze stability
    let performance = stability::analyze_stability(&bode_plot);

    Ok(PLLDesign {
        requirements: requirements.clone(),
        divider_r: r,
        divider_n,
        pfd_freq_hz,
        loop_filter: loop_filter_design,
        charge_pump_current_ua,
        vco_gain_mhz_per_v,
        performance,
        bode_plot,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_design_pll_basic() {
        let requirements = PLLRequirements {
            ref_freq_hz: 10e6,
            output_freq_min_hz: 2.4e9,
            output_freq_max_hz: 2.5e9,
            loop_bandwidth_hz: 100e3,
            phase_margin_deg: 45.0,
            architecture: PLLArchitecture::IntegerN,
            supply_voltage: 3.3,
        };

        let result = design_pll(&requirements);
        assert!(result.is_ok());

        let design = result.unwrap();
        assert_eq!(design.divider_r, 1);
        assert!(design.performance.phase_margin_deg > 0.0);
    }

    #[test]
    fn test_design_pll_validation() {
        let invalid_requirements = PLLRequirements {
            ref_freq_hz: 10e6,
            output_freq_min_hz: 2.5e9,
            output_freq_max_hz: 2.4e9,  // Invalid: min > max
            loop_bandwidth_hz: 100e3,
            phase_margin_deg: 45.0,
            architecture: PLLArchitecture::IntegerN,
            supply_voltage: 3.3,
        };

        let result = design_pll(&invalid_requirements);
        assert!(result.is_err());
    }
}
