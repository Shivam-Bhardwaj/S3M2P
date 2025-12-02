use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SPICE netlist representation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Netlist {
    pub title: String,
    pub elements: Vec<Element>,
    pub nodes: HashMap<String, usize>,  // Node name -> index mapping
    pub ground_node: String,
}

/// Circuit element types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Element {
    Resistor {
        name: String,
        node_p: String,
        node_n: String,
        value: f64,  // Ohms
    },
    Capacitor {
        name: String,
        node_p: String,
        node_n: String,
        value: f64,  // Farads
    },
    Inductor {
        name: String,
        node_p: String,
        node_n: String,
        value: f64,  // Henrys
    },
    VoltageSource {
        name: String,
        node_p: String,
        node_n: String,
        value: SourceValue,
    },
    CurrentSource {
        name: String,
        node_p: String,
        node_n: String,
        value: f64,  // Amps
    },
    VCVS {  // Voltage-Controlled Voltage Source
        name: String,
        node_out_p: String,
        node_out_n: String,
        node_ctrl_p: String,
        node_ctrl_n: String,
        gain: f64,
    },
    VCCS {  // Voltage-Controlled Current Source
        name: String,
        node_out_p: String,
        node_out_n: String,
        node_ctrl_p: String,
        node_ctrl_n: String,
        transconductance: f64,  // Siemens
    },
    /// Behavioral voltage source: V = f(time or other voltages)
    BehavioralV {
        name: String,
        node_p: String,
        node_n: String,
        expression: BehavioralExpression,
    },
    /// Behavioral current source: I = f(time or other voltages)
    BehavioralI {
        name: String,
        node_p: String,
        node_n: String,
        expression: BehavioralExpression,
    },
}

/// Source value types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SourceValue {
    DC(f64),
    AC { magnitude: f64, phase: f64 },
    Pulse {
        v1: f64,
        v2: f64,
        delay: f64,
        rise_time: f64,
        fall_time: f64,
        pulse_width: f64,
        period: f64,
    },
    Sin {
        offset: f64,
        amplitude: f64,
        freq: f64,
        delay: f64,
        damping: f64,
    },
}

/// Behavioral expressions for PLL components
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BehavioralExpression {
    /// Constant value
    Constant(f64),
    /// Linear: a*x + b
    Linear { a: f64, b: f64, input_node_p: String, input_node_n: String },
    /// VCO: V = V0 + Kvco * Vtune
    VCO { v_center: f64, kvco: f64, tune_node_p: String, tune_node_n: String },
    /// Phase detector: Vout = Kpd * Δφ (simplified)
    PhaseDetector { kpd: f64, ref_node: String, fb_node: String },
    /// Custom function (for future expansion)
    Custom { function_name: String, params: Vec<f64> },
}

impl Netlist {
    /// Create a new netlist
    pub fn new(title: String) -> Self {
        let mut nodes = HashMap::new();
        nodes.insert("0".to_string(), 0);  // Ground is always node 0

        Self {
            title,
            elements: Vec::new(),
            nodes,
            ground_node: "0".to_string(),
        }
    }

    /// Add an element to the netlist
    pub fn add_element(&mut self, element: Element) {
        // Register all nodes
        match &element {
            Element::Resistor { node_p, node_n, .. } |
            Element::Capacitor { node_p, node_n, .. } |
            Element::Inductor { node_p, node_n, .. } => {
                self.register_node(node_p);
                self.register_node(node_n);
            }
            Element::VoltageSource { node_p, node_n, .. } |
            Element::CurrentSource { node_p, node_n, .. } => {
                self.register_node(node_p);
                self.register_node(node_n);
            }
            Element::VCVS { node_out_p, node_out_n, node_ctrl_p, node_ctrl_n, .. } |
            Element::VCCS { node_out_p, node_out_n, node_ctrl_p, node_ctrl_n, .. } => {
                self.register_node(node_out_p);
                self.register_node(node_out_n);
                self.register_node(node_ctrl_p);
                self.register_node(node_ctrl_n);
            }
            Element::BehavioralV { node_p, node_n, expression, .. } |
            Element::BehavioralI { node_p, node_n, expression, .. } => {
                self.register_node(node_p);
                self.register_node(node_n);
                // Register expression nodes
                match expression {
                    BehavioralExpression::Linear { input_node_p, input_node_n, .. } => {
                        self.register_node(input_node_p);
                        self.register_node(input_node_n);
                    }
                    BehavioralExpression::VCO { tune_node_p, tune_node_n, .. } => {
                        self.register_node(tune_node_p);
                        self.register_node(tune_node_n);
                    }
                    _ => {}
                }
            }
        }

        self.elements.push(element);
    }

    /// Register a node and assign it an index
    fn register_node(&mut self, node: &str) {
        if !self.nodes.contains_key(node) {
            let next_idx = self.nodes.len();
            self.nodes.insert(node.to_string(), next_idx);
        }
    }

    /// Get node index
    pub fn node_index(&self, node: &str) -> Option<usize> {
        self.nodes.get(node).copied()
    }

    /// Get total number of nodes (excluding ground)
    pub fn num_nodes(&self) -> usize {
        self.nodes.len() - 1  // Exclude ground
    }

    /// Count voltage sources (for matrix sizing)
    pub fn num_voltage_sources(&self) -> usize {
        self.elements.iter().filter(|e| matches!(e,
            Element::VoltageSource { .. } |
            Element::VCVS { .. } |
            Element::BehavioralV { .. }
        )).count()
    }
}

impl Element {
    /// Get element name
    pub fn name(&self) -> &str {
        match self {
            Element::Resistor { name, .. } |
            Element::Capacitor { name, .. } |
            Element::Inductor { name, .. } |
            Element::VoltageSource { name, .. } |
            Element::CurrentSource { name, .. } |
            Element::VCVS { name, .. } |
            Element::VCCS { name, .. } |
            Element::BehavioralV { name, .. } |
            Element::BehavioralI { name, .. } => name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_netlist_creation() {
        let mut netlist = Netlist::new("Test Circuit".to_string());

        netlist.add_element(Element::Resistor {
            name: "R1".to_string(),
            node_p: "1".to_string(),
            node_n: "0".to_string(),
            value: 1000.0,
        });

        assert_eq!(netlist.num_nodes(), 1);  // Node 1 (ground doesn't count)
        assert_eq!(netlist.elements.len(), 1);
        assert_eq!(netlist.node_index("0"), Some(0));
        assert_eq!(netlist.node_index("1"), Some(1));
    }

    #[test]
    fn test_voltage_source_count() {
        let mut netlist = Netlist::new("Test".to_string());

        netlist.add_element(Element::VoltageSource {
            name: "V1".to_string(),
            node_p: "1".to_string(),
            node_n: "0".to_string(),
            value: SourceValue::DC(5.0),
        });

        netlist.add_element(Element::Resistor {
            name: "R1".to_string(),
            node_p: "1".to_string(),
            node_n: "0".to_string(),
            value: 1000.0,
        });

        assert_eq!(netlist.num_voltage_sources(), 1);
    }
}
