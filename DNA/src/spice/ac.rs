use super::netlist::{Element, Netlist, SourceValue};
use std::f64::consts::PI;

/// Complex number for AC analysis
#[derive(Clone, Copy, Debug)]
pub struct Complex {
    pub real: f64,
    pub imag: f64,
}

impl Complex {
    pub fn new(real: f64, imag: f64) -> Self {
        Self { real, imag }
    }

    pub fn zero() -> Self {
        Self { real: 0.0, imag: 0.0 }
    }

    pub fn from_polar(magnitude: f64, phase_rad: f64) -> Self {
        Self {
            real: magnitude * phase_rad.cos(),
            imag: magnitude * phase_rad.sin(),
        }
    }

    pub fn magnitude(&self) -> f64 {
        (self.real * self.real + self.imag * self.imag).sqrt()
    }

    pub fn phase_rad(&self) -> f64 {
        self.imag.atan2(self.real)
    }

    pub fn phase_deg(&self) -> f64 {
        self.phase_rad() * 180.0 / PI
    }

    pub fn conjugate(&self) -> Complex {
        Complex::new(self.real, -self.imag)
    }
}

impl std::ops::Add for Complex {
    type Output = Complex;
    fn add(self, other: Complex) -> Complex {
        Complex::new(self.real + other.real, self.imag + other.imag)
    }
}

impl std::ops::Sub for Complex {
    type Output = Complex;
    fn sub(self, other: Complex) -> Complex {
        Complex::new(self.real - other.real, self.imag - other.imag)
    }
}

impl std::ops::Mul for Complex {
    type Output = Complex;
    fn mul(self, other: Complex) -> Complex {
        Complex::new(
            self.real * other.real - self.imag * other.imag,
            self.real * other.imag + self.imag * other.real,
        )
    }
}

impl std::ops::Mul<f64> for Complex {
    type Output = Complex;
    fn mul(self, scalar: f64) -> Complex {
        Complex::new(self.real * scalar, self.imag * scalar)
    }
}

impl std::ops::Div for Complex {
    type Output = Complex;
    fn div(self, other: Complex) -> Complex {
        let denom = other.real * other.real + other.imag * other.imag;
        Complex::new(
            (self.real * other.real + self.imag * other.imag) / denom,
            (self.imag * other.real - self.real * other.imag) / denom,
        )
    }
}

/// Complex MNA matrix for AC analysis
#[derive(Clone, Debug)]
pub struct ComplexMNAMatrix {
    pub size: usize,
    pub num_nodes: usize,
    pub num_vsources: usize,
    pub matrix: Vec<Vec<Complex>>,
    pub rhs: Vec<Complex>,
}

impl ComplexMNAMatrix {
    pub fn new(num_nodes: usize, num_vsources: usize) -> Self {
        let size = num_nodes + num_vsources;
        let matrix = vec![vec![Complex::zero(); size]; size];
        let rhs = vec![Complex::zero(); size];

        Self {
            size,
            num_nodes,
            num_vsources,
            matrix,
            rhs,
        }
    }

    /// Stamp resistor (frequency-independent)
    pub fn stamp_resistor(&mut self, node_p: usize, node_n: usize, resistance: f64) {
        let g = Complex::new(1.0 / resistance, 0.0);

        if node_p > 0 {
            self.matrix[node_p - 1][node_p - 1] = self.matrix[node_p - 1][node_p - 1] + g;
            if node_n > 0 {
                self.matrix[node_p - 1][node_n - 1] = self.matrix[node_p - 1][node_n - 1] - g;
            }
        }

        if node_n > 0 {
            self.matrix[node_n - 1][node_n - 1] = self.matrix[node_n - 1][node_n - 1] + g;
            if node_p > 0 {
                self.matrix[node_n - 1][node_p - 1] = self.matrix[node_n - 1][node_p - 1] - g;
            }
        }
    }

    /// Stamp capacitor: Y = jωC
    pub fn stamp_capacitor(&mut self, node_p: usize, node_n: usize, capacitance: f64, omega: f64) {
        let y = Complex::new(0.0, omega * capacitance);

        if node_p > 0 {
            self.matrix[node_p - 1][node_p - 1] = self.matrix[node_p - 1][node_p - 1] + y;
            if node_n > 0 {
                self.matrix[node_p - 1][node_n - 1] = self.matrix[node_p - 1][node_n - 1] - y;
            }
        }

        if node_n > 0 {
            self.matrix[node_n - 1][node_n - 1] = self.matrix[node_n - 1][node_n - 1] + y;
            if node_p > 0 {
                self.matrix[node_n - 1][node_p - 1] = self.matrix[node_n - 1][node_p - 1] - y;
            }
        }
    }

    /// Stamp inductor: Y = 1/(jωL)
    pub fn stamp_inductor(&mut self, node_p: usize, node_n: usize, inductance: f64, omega: f64) {
        if omega == 0.0 {
            return;  // DC: inductor is short circuit
        }

        let y = Complex::new(0.0, -1.0 / (omega * inductance));

        if node_p > 0 {
            self.matrix[node_p - 1][node_p - 1] = self.matrix[node_p - 1][node_p - 1] + y;
            if node_n > 0 {
                self.matrix[node_p - 1][node_n - 1] = self.matrix[node_p - 1][node_n - 1] - y;
            }
        }

        if node_n > 0 {
            self.matrix[node_n - 1][node_n - 1] = self.matrix[node_n - 1][node_n - 1] + y;
            if node_p > 0 {
                self.matrix[node_n - 1][node_p - 1] = self.matrix[node_n - 1][node_p - 1] - y;
            }
        }
    }

    /// Stamp voltage source
    pub fn stamp_voltage_source(&mut self, node_p: usize, node_n: usize, vs_idx: usize, voltage: Complex) {
        let vs_row = self.num_nodes + vs_idx;

        if node_p > 0 {
            self.matrix[node_p - 1][vs_row] = self.matrix[node_p - 1][vs_row] + Complex::new(1.0, 0.0);
            self.matrix[vs_row][node_p - 1] = self.matrix[vs_row][node_p - 1] + Complex::new(1.0, 0.0);
        }

        if node_n > 0 {
            self.matrix[node_n - 1][vs_row] = self.matrix[node_n - 1][vs_row] - Complex::new(1.0, 0.0);
            self.matrix[vs_row][node_n - 1] = self.matrix[vs_row][node_n - 1] - Complex::new(1.0, 0.0);
        }

        self.rhs[vs_row] = voltage;
    }

    /// Stamp VCCS (transconductance)
    pub fn stamp_vccs(
        &mut self,
        node_out_p: usize,
        node_out_n: usize,
        node_ctrl_p: usize,
        node_ctrl_n: usize,
        transconductance: f64,
    ) {
        let gm = Complex::new(transconductance, 0.0);

        if node_out_p > 0 && node_ctrl_p > 0 {
            self.matrix[node_out_p - 1][node_ctrl_p - 1] =
                self.matrix[node_out_p - 1][node_ctrl_p - 1] + gm;
        }
        if node_out_p > 0 && node_ctrl_n > 0 {
            self.matrix[node_out_p - 1][node_ctrl_n - 1] =
                self.matrix[node_out_p - 1][node_ctrl_n - 1] - gm;
        }
        if node_out_n > 0 && node_ctrl_p > 0 {
            self.matrix[node_out_n - 1][node_ctrl_p - 1] =
                self.matrix[node_out_n - 1][node_ctrl_p - 1] - gm;
        }
        if node_out_n > 0 && node_ctrl_n > 0 {
            self.matrix[node_out_n - 1][node_ctrl_n - 1] =
                self.matrix[node_out_n - 1][node_ctrl_n - 1] + gm;
        }
    }

    /// Stamp VCVS (voltage-controlled voltage source)
    /// This requires adding an auxiliary variable for the current through the VCVS
    pub fn stamp_vcvs(
        &mut self,
        node_out_p: usize,
        node_out_n: usize,
        node_ctrl_p: usize,
        node_ctrl_n: usize,
        gain: f64,
        vs_index: usize,
    ) {
        let vs_row = self.num_nodes + vs_index;

        // KCL at output nodes (current flows through VCVS)
        if node_out_p > 0 {
            self.matrix[node_out_p - 1][vs_row] = self.matrix[node_out_p - 1][vs_row] + Complex::new(1.0, 0.0);
            self.matrix[vs_row][node_out_p - 1] = self.matrix[vs_row][node_out_p - 1] + Complex::new(1.0, 0.0);
        }
        if node_out_n > 0 {
            self.matrix[node_out_n - 1][vs_row] = self.matrix[node_out_n - 1][vs_row] - Complex::new(1.0, 0.0);
            self.matrix[vs_row][node_out_n - 1] = self.matrix[vs_row][node_out_n - 1] - Complex::new(1.0, 0.0);
        }

        // Voltage constraint: V_out = gain * V_ctrl
        // V_out_p - V_out_n - gain * (V_ctrl_p - V_ctrl_n) = 0
        let g = Complex::new(gain, 0.0);
        if node_ctrl_p > 0 {
            self.matrix[vs_row][node_ctrl_p - 1] = self.matrix[vs_row][node_ctrl_p - 1] - g;
        }
        if node_ctrl_n > 0 {
            self.matrix[vs_row][node_ctrl_n - 1] = self.matrix[vs_row][node_ctrl_n - 1] + g;
        }
    }

    /// Solve using complex LU decomposition
    pub fn solve(&self) -> Result<Vec<Complex>, String> {
        let mut a = self.matrix.clone();
        let b = self.rhs.clone();
        let n = self.size;

        // LU decomposition
        for k in 0..n {
            if a[k][k].magnitude() < 1e-12 {
                return Err(format!("Matrix is singular at row {}", k));
            }

            for i in (k + 1)..n {
                let factor = a[i][k] / a[k][k];
                a[i][k] = factor;

                for j in (k + 1)..n {
                    a[i][j] = a[i][j] - factor * a[k][j];
                }
            }
        }

        // Forward substitution
        let mut y = vec![Complex::zero(); n];
        for i in 0..n {
            let mut sum = b[i];
            for j in 0..i {
                sum = sum - a[i][j] * y[j];
            }
            y[i] = sum;
        }

        // Back substitution
        let mut x = vec![Complex::zero(); n];
        for i in (0..n).rev() {
            let mut sum = y[i];
            for j in (i + 1)..n {
                sum = sum - a[i][j] * x[j];
            }
            x[i] = sum / a[i][i];
        }

        Ok(x)
    }
}

/// AC analysis result
#[derive(Clone, Debug)]
pub struct ACResult {
    pub frequencies: Vec<f64>,
    pub node_voltages: Vec<Vec<Complex>>,  // [frequency][node]
}

/// Perform AC analysis
pub fn ac_analysis(
    netlist: &Netlist,
    freq_start: f64,
    freq_stop: f64,
    points_per_decade: usize,
) -> Result<ACResult, String> {
    let num_nodes = netlist.num_nodes();
    let num_vsources = netlist.num_voltage_sources();

    let mut frequencies = Vec::new();
    let mut node_voltages = Vec::new();

    // Generate frequency points (logarithmic)
    let start_log = freq_start.log10();
    let stop_log = freq_stop.log10();
    let decades = stop_log - start_log;
    let num_points = (decades * points_per_decade as f64).ceil() as usize;

    for i in 0..num_points {
        let log_freq = start_log + (i as f64 / (num_points - 1) as f64) * (stop_log - start_log);
        let freq = 10.0_f64.powf(log_freq);
        let omega = 2.0 * PI * freq;

        // Build matrix for this frequency
        let mut matrix = ComplexMNAMatrix::new(num_nodes, num_vsources);
        let mut vs_count = 0;

        for element in &netlist.elements {
            match element {
                Element::Resistor { node_p, node_n, value, .. } => {
                    let np = netlist.node_index(node_p).unwrap();
                    let nn = netlist.node_index(node_n).unwrap();
                    matrix.stamp_resistor(np, nn, *value);
                }
                Element::Capacitor { node_p, node_n, value, .. } => {
                    let np = netlist.node_index(node_p).unwrap();
                    let nn = netlist.node_index(node_n).unwrap();
                    matrix.stamp_capacitor(np, nn, *value, omega);
                }
                Element::Inductor { node_p, node_n, value, .. } => {
                    let np = netlist.node_index(node_p).unwrap();
                    let nn = netlist.node_index(node_n).unwrap();
                    matrix.stamp_inductor(np, nn, *value, omega);
                }
                Element::VoltageSource { node_p, node_n, value, .. } => {
                    let np = netlist.node_index(node_p).unwrap();
                    let nn = netlist.node_index(node_n).unwrap();

                    let v_complex = match value {
                        SourceValue::DC(v) => Complex::new(*v, 0.0),
                        SourceValue::AC { magnitude, phase } => {
                            Complex::from_polar(*magnitude, phase * PI / 180.0)
                        }
                        _ => Complex::zero(),
                    };

                    matrix.stamp_voltage_source(np, nn, vs_count, v_complex);
                    vs_count += 1;
                }
                Element::VCCS { node_out_p, node_out_n, node_ctrl_p, node_ctrl_n, transconductance, .. } => {
                    let nop = netlist.node_index(node_out_p).unwrap();
                    let non = netlist.node_index(node_out_n).unwrap();
                    let ncp = netlist.node_index(node_ctrl_p).unwrap();
                    let ncn = netlist.node_index(node_ctrl_n).unwrap();
                    matrix.stamp_vccs(nop, non, ncp, ncn, *transconductance);
                }
                Element::VCVS { node_out_p, node_out_n, node_ctrl_p, node_ctrl_n, gain, .. } => {
                    let nop = netlist.node_index(node_out_p).unwrap();
                    let non = netlist.node_index(node_out_n).unwrap();
                    let ncp = netlist.node_index(node_ctrl_p).unwrap();
                    let ncn = netlist.node_index(node_ctrl_n).unwrap();
                    matrix.stamp_vcvs(nop, non, ncp, ncn, *gain, vs_count);
                    vs_count += 1;
                }
                _ => {}
            }
        }

        // Solve
        let solution = matrix.solve()?;

        frequencies.push(freq);
        node_voltages.push(solution);
    }

    Ok(ACResult {
        frequencies,
        node_voltages,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complex_arithmetic() {
        let a = Complex::new(3.0, 4.0);
        let b = Complex::new(1.0, 2.0);

        assert!((a.magnitude() - 5.0).abs() < 1e-10);

        let c = a + b;
        assert!((c.real - 4.0).abs() < 1e-10);
        assert!((c.imag - 6.0).abs() < 1e-10);

        let d = a * b;
        assert!((d.real - -5.0).abs() < 1e-10);
        assert!((d.imag - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_rc_lowpass() {
        // RC lowpass: V_in -> R (1k) -> node 1 -> C (1µF) -> GND
        // Cutoff frequency = 1/(2πRC) = 159 Hz

        let mut netlist = Netlist::new("RC Lowpass".to_string());

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

        let result = ac_analysis(&netlist, 10.0, 10000.0, 20).unwrap();

        // At low frequency, output should be close to input
        let v_out_low = result.node_voltages[0][1];  // Node "out" at first frequency
        assert!(v_out_low.magnitude() > 0.9);

        // At high frequency, output should be attenuated
        let v_out_high = result.node_voltages[result.node_voltages.len() - 1][1];
        assert!(v_out_high.magnitude() < 0.1);
    }
}
