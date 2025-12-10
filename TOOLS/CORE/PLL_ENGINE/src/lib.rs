//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: lib.rs | TOOLS/CORE/PLL_ENGINE/src/lib.rs
//! PURPOSE: PLL (Phase-Locked Loop) design automation engine
//! MODIFIED: 2025-12-09
//! LAYER: CORE → PLL_ENGINE
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! PLL_ENGINE automates PLL design and analysis:
//! - Loop filter design (passive, active)
//! - Stability analysis (phase margin, gain margin)
//! - Transient simulation (lock time, overshoot)
//! - Noise analysis (phase noise, jitter)
//!
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │ ARCHITECTURE                                                                │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │   PLLEngine                                                                 │
//! │       │                                                                     │
//! │       ├── PLLRequirements      (DNA/pll/types)                              │
//! │       ├── PLLDesign            (DNA/pll/types)                              │
//! │       ├── PLLPerformance       (DNA/pll/types)                              │
//! │       └── LoopFilterDesign     (DNA/pll/loop_filter)                        │
//! │                                                                             │
//! │   Design flow:                                                              │
//! │   1. Specify VCO, reference, output frequency                               │
//! │   2. Calculate divider ratios                                               │
//! │   3. Design loop filter for desired bandwidth/phase margin                  │
//! │   4. Verify stability                                                       │
//! │   5. Simulate transient response                                            │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//!
//! DEPENDS ON:
//!   • DNA/pll → PLL design types and algorithms
//!   • CORE/SPICE_ENGINE → Circuit simulation
//!
//! USED BY:
//!   • TOOLS/PLL → Interactive PLL designer
//!
//! ═══════════════════════════════════════════════════════════════════════════════

// ─────────────────────────────────────────────────────────────────────────────────
// CODE BELOW - Optimized for ML development
// ─────────────────────────────────────────────────────────────────────────────────

// Re-export PLL types from DNA
pub use dna::pll::{
    // Main design function
    design_pll,
    // Bode plot and noise
    BodePlot,
    // Divider config
    DividerConfig,
    // Loop filter
    LoopFilterDesign,
    LoopFilterTopology,
    PLLArchitecture,
    PLLDesign,
    PLLPerformance,
    // Main types
    PLLRequirements,
    PhaseNoiseProfile,
    // Transient simulation
    TransientResult,
};

// Re-export SPICE engine for circuit-level simulation
pub use spice_engine::{ac_analysis, ACResult, Element, Netlist, SourceValue};

/// Design a PLL with simplified interface
///
/// This is a convenience wrapper around `design_pll` for common use cases.
pub fn quick_design_integer_n(
    f_ref: f64,
    f_out: f64,
    loop_bandwidth: f64,
    phase_margin: f64,
) -> Result<PLLDesign, String> {
    let requirements = PLLRequirements {
        ref_freq_hz: f_ref,
        output_freq_min_hz: f_out * 0.99, // Allow 1% range
        output_freq_max_hz: f_out * 1.01,
        loop_bandwidth_hz: loop_bandwidth,
        phase_margin_deg: phase_margin,
        architecture: PLLArchitecture::IntegerN,
        supply_voltage: 3.3,
    };

    design_pll(&requirements)
}

/// Check if a PLL design meets stability requirements
pub fn check_stability(design: &PLLDesign) -> bool {
    design.performance.phase_margin_deg >= 30.0 && design.performance.gain_margin_db >= 6.0
}

/// Get a summary of PLL performance metrics
pub fn performance_summary(design: &PLLDesign) -> String {
    let is_stable = check_stability(design);
    format!(
        "Phase Margin: {:.1}°, Gain Margin: {:.1} dB, Lock Time: {:.1} µs, Stable: {}",
        design.performance.phase_margin_deg,
        design.performance.gain_margin_db,
        design.performance.lock_time_us,
        if is_stable { "Yes" } else { "No" }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quick_design() {
        let result = quick_design_integer_n(
            10e6,  // 10 MHz reference
            1e9,   // 1 GHz output
            100e3, // 100 kHz loop bandwidth
            45.0,  // 45 degree phase margin
        );

        assert!(result.is_ok());
        let design = result.unwrap();

        // Should have positive phase margin
        assert!(design.performance.phase_margin_deg > 0.0);
    }

    #[test]
    fn test_check_stability() {
        let result = quick_design_integer_n(10e6, 1e9, 100e3, 45.0);
        assert!(result.is_ok());
        let design = result.unwrap();

        let is_stable = check_stability(&design);
        // A well-designed PLL should be stable
        assert!(is_stable);
    }

    #[test]
    fn test_performance_summary() {
        let result = quick_design_integer_n(10e6, 1e9, 100e3, 45.0);
        assert!(result.is_ok());
        let design = result.unwrap();

        let summary = performance_summary(&design);
        assert!(summary.contains("Phase Margin"));
        assert!(summary.contains("Gain Margin"));
    }
}
