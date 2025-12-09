//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: lib.rs | CORE/SPICE_ENGINE/src/lib.rs
//! PURPOSE: Circuit simulation engine (DC, AC, Transient analysis)
//! LAYER: CORE → SPICE_ENGINE
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! SPICE_ENGINE provides circuit simulation capabilities:
//! - DC analysis (operating point)
//! - AC analysis (frequency response, Bode plots)
//! - Transient analysis (time-domain simulation) [TODO]
//!
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │ ARCHITECTURE                                                                │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │   SpiceEngine                                                               │
//! │       │                                                                     │
//! │       ├── Netlist              (DNA/physics/electromagnetics/lumped)        │
//! │       ├── MNAMatrix            (DNA/physics/electromagnetics/lumped)        │
//! │       ├── ComplexMNAMatrix     (DNA/physics/electromagnetics/lumped)        │
//! │       └── ACResult             (DNA/physics/electromagnetics/lumped)        │
//! │                                                                             │
//! │   Analysis types:                                                           │
//! │   - dc_analysis()   - Find DC operating point                               │
//! │   - ac_analysis()   - Frequency sweep with complex arithmetic               │
//! │   - bode_plot()     - Generate magnitude/phase vs frequency                 │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//!
//! DEPENDS ON:
//!   • DNA/physics/electromagnetics/lumped → Netlist, MNA matrices
//!   • DNA/physics/electromagnetics/lumped/ac → Complex numbers, AC analysis
//!
//! USED BY:
//!   • TOOLS/PLL → PLL frequency response
//!   • TOOLS/POWER_CIRCUITS → Power supply analysis
//!
//! ═══════════════════════════════════════════════════════════════════════════════

// ─────────────────────────────────────────────────────────────────────────────────
// CODE BELOW - Optimized for ML development
// ─────────────────────────────────────────────────────────────────────────────────

// Re-export lumped circuit types from DNA
pub use dna::physics::electromagnetics::lumped::{
    // Netlist types
    Netlist, Element, SourceValue, BehavioralExpression,
    // Matrix types
    MNAMatrix, ComplexMNAMatrix, Complex,
    // Analysis functions
    ac_analysis, ACResult,
};

/// Bode plot data point
#[derive(Clone, Debug)]
pub struct BodePoint {
    pub frequency: f64,
    pub magnitude_db: f64,
    pub phase_deg: f64,
}

/// Generate Bode plot data from AC analysis result
pub fn generate_bode_plot(ac_result: &ACResult, output_node: usize) -> Vec<BodePoint> {
    ac_result.frequencies
        .iter()
        .zip(ac_result.node_voltages.iter())
        .map(|(&freq, voltages)| {
            let v = voltages.get(output_node).copied().unwrap_or(Complex::zero());
            BodePoint {
                frequency: freq,
                magnitude_db: 20.0 * v.magnitude().log10(),
                phase_deg: v.phase_deg(),
            }
        })
        .collect()
}

/// Find -3dB cutoff frequency from Bode data
pub fn find_cutoff_frequency(bode: &[BodePoint]) -> Option<f64> {
    if bode.is_empty() {
        return None;
    }

    let dc_gain = bode[0].magnitude_db;
    let cutoff_level = dc_gain - 3.0;

    for i in 1..bode.len() {
        if bode[i].magnitude_db <= cutoff_level {
            // Linear interpolation
            let f0 = bode[i - 1].frequency;
            let f1 = bode[i].frequency;
            let m0 = bode[i - 1].magnitude_db;
            let m1 = bode[i].magnitude_db;

            let t = (cutoff_level - m0) / (m1 - m0);
            return Some(f0 + t * (f1 - f0));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bode_generation() {
        let mut netlist = Netlist::new("RC Filter".to_string());

        netlist.add_element(Element::VoltageSource {
            name: "V1".to_string(),
            node_p: "in".to_string(),
            node_n: "0".to_string(),
            value: SourceValue::AC { magnitude: 1.0, phase: 0.0 },
        });

        netlist.add_element(Element::Resistor {
            name: "R1".to_string(),
            node_p: "in".to_string(),
            node_n: "out".to_string(),
            value: 1000.0,
        });

        netlist.add_element(Element::Capacitor {
            name: "C1".to_string(),
            node_p: "out".to_string(),
            node_n: "0".to_string(),
            value: 1e-6,
        });

        let ac_result = ac_analysis(&netlist, 1.0, 100000.0, 20).unwrap();
        let bode = generate_bode_plot(&ac_result, 1);

        // Should have multiple frequency points
        assert!(bode.len() > 10);

        // Low frequency should have ~0dB gain
        assert!(bode[0].magnitude_db.abs() < 1.0);

        // High frequency should be attenuated
        let last = &bode[bode.len() - 1];
        assert!(last.magnitude_db < -20.0);
    }

    #[test]
    fn test_cutoff_frequency() {
        let bode = vec![
            BodePoint { frequency: 10.0, magnitude_db: 0.0, phase_deg: 0.0 },
            BodePoint { frequency: 100.0, magnitude_db: -2.0, phase_deg: -30.0 },
            BodePoint { frequency: 200.0, magnitude_db: -4.0, phase_deg: -50.0 },
        ];

        let cutoff = find_cutoff_frequency(&bode);
        assert!(cutoff.is_some());
        let fc = cutoff.unwrap();
        // Should be between 100 and 200 Hz
        assert!(fc > 100.0 && fc < 200.0);
    }
}
