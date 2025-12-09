use crate::physics::electromagnetics::lumped::*;
use crate::pll::*;
use std::f64::consts::PI;

/// Build a SPICE netlist for the PLL circuit
///
/// This creates a linearized small-signal model of the PLL:
/// - Phase-Frequency Detector (PFD) + Charge Pump → Current source (Icp)
/// - Loop Filter → R-C network
/// - VCO → Voltage-controlled current source (models 1/s in Laplace domain)
/// - Feedback Divider → Gain block (1/N)
///
/// The open-loop transfer function is:
/// ```text
/// G(s) = K_phi * Z(s) * K_vco / (s * N)
/// ```
///
/// Where Z(s) is the loop filter impedance.
pub fn build_pll_netlist(design: &PLLDesign) -> Netlist {
    let mut netlist = Netlist::new("PLL Open-Loop".to_string());

    // Extract parameters
    let k_phi = design.charge_pump_current_ua * 1e-6; // A
    let k_vco = design.vco_gain_mhz_per_v * 1e6 * 2.0 * PI; // rad/s/V
    let n = match &design.divider_n {
        DividerConfig::IntegerN { n, .. } => *n as f64,
        DividerConfig::FractionalN { n_int, .. } => *n_int as f64,
    };

    let r1 = design.loop_filter.r1_ohms;
    let c1 = design.loop_filter.c1_pf * 1e-12; // Convert pF to F
    let c2 = design.loop_filter.c2_pf * 1e-12;

    // Input: AC voltage source representing phase error (rad)
    // The PFD+CP converts this to current: I = K_phi * phase_error
    netlist.add_element(Element::VoltageSource {
        name: "V_phase".to_string(),
        node_p: "phase_in".to_string(),
        node_n: "0".to_string(),
        value: SourceValue::AC {
            magnitude: 1.0,
            phase: 0.0,
        },
    });

    // PFD + Charge Pump: Voltage-controlled current source
    // I_cp = K_phi * V_phase
    netlist.add_element(Element::VCCS {
        name: "G_pfd_cp".to_string(),
        node_out_p: "filter_in".to_string(),
        node_out_n: "0".to_string(),
        node_ctrl_p: "phase_in".to_string(),
        node_ctrl_n: "0".to_string(),
        transconductance: k_phi,
    });

    // Loop Filter: Second-order passive (C1-R1-C2)
    //
    //  filter_in ---+--- C1 --- 0
    //               |
    //               +--- R1 ---+--- filter_out
    //                          |
    //                          +--- C2 --- 0
    //
    netlist.add_element(Element::Capacitor {
        name: "C1".to_string(),
        node_p: "filter_in".to_string(),
        node_n: "0".to_string(),
        value: c1,
    });

    netlist.add_element(Element::Resistor {
        name: "R1".to_string(),
        node_p: "filter_in".to_string(),
        node_n: "filter_out".to_string(),
        value: r1,
    });

    netlist.add_element(Element::Capacitor {
        name: "C2".to_string(),
        node_p: "filter_out".to_string(),
        node_n: "0".to_string(),
        value: c2,
    });

    // VCO: In Laplace domain, VCO is K_vco/s
    // For AC analysis, we model this as:
    //   - A VCVS with gain K_vco (to get K_vco * V_ctrl)
    //   - Followed by an integrator (capacitor to ground)
    //
    // Actually, for open-loop frequency response, we want:
    //   Output_phase(s) = (K_vco/s) * V_ctrl(s)
    //
    // In frequency domain: Output_phase(jω) = (K_vco/jω) * V_ctrl(jω)
    //
    // We can model this as a VCCS into a capacitor:
    //   I = K_vco * V_ctrl
    //   V_out = I / (jω * C_integrate)
    //
    // So: V_out = K_vco * V_ctrl / (jω * C_integrate)
    //
    // Set C_integrate = 1 F for simplicity, giving V_out = K_vco/(jω) * V_ctrl
    let c_integrate = 1.0; // 1 Farad

    netlist.add_element(Element::VCCS {
        name: "G_vco".to_string(),
        node_out_p: "vco_out".to_string(),
        node_out_n: "0".to_string(),
        node_ctrl_p: "filter_out".to_string(),
        node_ctrl_n: "0".to_string(),
        transconductance: k_vco,
    });

    netlist.add_element(Element::Capacitor {
        name: "C_int".to_string(),
        node_p: "vco_out".to_string(),
        node_n: "0".to_string(),
        value: c_integrate,
    });

    // Feedback Divider: VCVS with gain 1/N
    netlist.add_element(Element::VCVS {
        name: "E_div".to_string(),
        node_out_p: "feedback".to_string(),
        node_out_n: "0".to_string(),
        node_ctrl_p: "vco_out".to_string(),
        node_ctrl_n: "0".to_string(),
        gain: 1.0 / n,
    });

    // Add high-resistance load to feedback node to avoid floating node
    // (For open-loop analysis, this node isn't connected back to input)
    netlist.add_element(Element::Resistor {
        name: "R_load".to_string(),
        node_p: "feedback".to_string(),
        node_n: "0".to_string(),
        value: 1e12, // 1 TOhm - effectively open but avoids singularity
    });

    netlist
}

/// Run SPICE AC analysis on the PLL circuit
///
/// Returns Bode plot data computed from SPICE simulation
pub fn simulate_pll_circuit(design: &PLLDesign) -> Result<BodePlot, String> {
    let netlist = build_pll_netlist(design);

    // Frequency sweep: 1 kHz to 10x reference frequency
    let freq_start = 1e3;
    let freq_stop = 10.0 * design.requirements.ref_freq_hz;
    let points_per_decade = 50;

    // Run AC analysis
    let ac_result = ac_analysis(&netlist, freq_start, freq_stop, points_per_decade)?;

    // Get node index for vco_out
    let vco_out_node = netlist
        .node_index("vco_out")
        .ok_or("vco_out node not found")?;

    // Extract open-loop transfer function: vco_out / phase_in
    let mut frequencies_hz = Vec::new();
    let mut magnitude_db = Vec::new();
    let mut phase_deg = Vec::new();

    for (i, &freq) in ac_result.frequencies.iter().enumerate() {
        frequencies_hz.push(freq);

        // Get vco_out voltage (this is the output phase in our model)
        // node_voltages is indexed [frequency][node]
        let vco_out = if vco_out_node > 0 && vco_out_node <= ac_result.node_voltages[i].len() {
            ac_result.node_voltages[i][vco_out_node - 1]
        } else {
            Complex::zero()
        };

        // Open-loop gain magnitude (input was 1V AC)
        let magnitude = vco_out.magnitude();
        magnitude_db.push(20.0 * magnitude.log10());

        // Phase in degrees
        let phase = vco_out.phase_deg();
        phase_deg.push(phase);
    }

    Ok(BodePlot {
        frequencies_hz,
        magnitude_db,
        phase_deg,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_pll_netlist() {
        let requirements = PLLRequirements {
            ref_freq_hz: 10e6,
            output_freq_min_hz: 2.4e9,
            output_freq_max_hz: 2.5e9,
            loop_bandwidth_hz: 100e3,
            phase_margin_deg: 45.0,
            architecture: PLLArchitecture::IntegerN,
            supply_voltage: 3.3,
        };

        let design = design_pll(&requirements).unwrap();
        let netlist = build_pll_netlist(&design);

        // Should have all the key components
        assert!(netlist
            .elements
            .iter()
            .any(|e| matches!(e, Element::VoltageSource { name, .. } if name == "V_phase")));
        assert!(netlist
            .elements
            .iter()
            .any(|e| matches!(e, Element::VCCS { name, .. } if name == "G_pfd_cp")));
        assert!(netlist
            .elements
            .iter()
            .any(|e| matches!(e, Element::Capacitor { name, .. } if name == "C1")));
        assert!(netlist
            .elements
            .iter()
            .any(|e| matches!(e, Element::Resistor { name, .. } if name == "R1")));
        assert!(netlist
            .elements
            .iter()
            .any(|e| matches!(e, Element::Capacitor { name, .. } if name == "C2")));
    }

    #[test]
    fn test_simulate_pll_circuit() {
        let requirements = PLLRequirements {
            ref_freq_hz: 10e6,
            output_freq_min_hz: 2.4e9,
            output_freq_max_hz: 2.5e9,
            loop_bandwidth_hz: 100e3,
            phase_margin_deg: 45.0,
            architecture: PLLArchitecture::IntegerN,
            supply_voltage: 3.3,
        };

        let design = design_pll(&requirements).unwrap();
        let bode = simulate_pll_circuit(&design);

        if let Err(ref e) = bode {
            eprintln!("SPICE simulation error: {}", e);
        }
        assert!(bode.is_ok());
        let bode = bode.unwrap();

        // Should have frequency points
        assert!(bode.frequencies_hz.len() > 0);
        assert_eq!(bode.frequencies_hz.len(), bode.magnitude_db.len());
        assert_eq!(bode.frequencies_hz.len(), bode.phase_deg.len());

        // First frequency should be around 1 kHz
        assert!(bode.frequencies_hz[0] >= 1e3);
    }
}
