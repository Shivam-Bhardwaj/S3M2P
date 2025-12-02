// SPICE simulation capabilities
//
// This module provides a lightweight, in-browser SPICE engine for
// circuit simulation. It implements Modified Nodal Analysis (MNA)
// for accurate circuit solving.
//
// ## Capabilities
//
// - **AC Analysis**: Frequency response, Bode plots
// - **Linear Components**: R, L, C
// - **Sources**: Voltage/current sources (DC, AC, pulse, sin)
// - **Controlled Sources**: VCVS, VCCS
// - **Behavioral Models**: Custom expressions for PLL components
//
// ## Example
//
// ```rust
// use dna::spice::*;
//
// let mut netlist = Netlist::new("RC Lowpass".to_string());
//
// netlist.add_element(Element::VoltageSource {
//     name: "V1".to_string(),
//     node_p: "in".to_string(),
//     node_n: "0".to_string(),
//     value: SourceValue::AC { magnitude: 1.0, phase: 0.0 },
// });
//
// netlist.add_element(Element::Resistor {
//     name: "R1".to_string(),
//     node_p: "in".to_string(),
//     node_n: "out".to_string(),
//     value: 1000.0,
// });
//
// netlist.add_element(Element::Capacitor {
//     name: "C1".to_string(),
//     node_p: "out".to_string(),
//     node_n: "0".to_string(),
//     value: 1e-6,
// });
//
// let result = ac_analysis(&netlist, 1.0, 100e3, 50).unwrap();
// ```
//
// ## Technical Details
//
// ### Modified Nodal Analysis (MNA)
//
// The MNA formulation creates a system of linear equations:
// ```text
// [G + sC] [V] = [I]
// ```
//
// Where:
// - G: Conductance matrix
// - C: Capacitance matrix
// - s: Complex frequency (jω for AC analysis)
// - V: Node voltages (unknowns)
// - I: Current sources
//
// For voltage sources, auxiliary variables are added:
// ```text
// [  G   B ] [ V ] = [ I ]
// [ B^T  0 ] [ J ]   [ E ]
// ```
//
// Where J is the current through voltage sources.
//
// ### Numerical Methods
//
// - **LU Decomposition**: With partial pivoting for numerical stability
// - **Complex Arithmetic**: Full complex number support for AC analysis
// - **Frequency Sweep**: Logarithmic spacing for Bode plots
//
// ### Performance
//
// - O(n³) solve time (n = number of nodes + voltage sources)
// - Zero-allocation after matrix setup (reuses buffers)
// - Suitable for circuits with 10-1000 nodes in-browser
//
// ## Limitations
//
// - No nonlinear components (diodes, transistors) - use small-signal models
// - No time-domain analysis yet (transient coming soon)
// - No convergence checking for DC operating point
pub mod ac;
pub mod matrix;
pub mod netlist;

pub use ac::*;
pub use matrix::*;
pub use netlist::*;
