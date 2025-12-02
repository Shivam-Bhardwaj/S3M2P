/// Modified Nodal Analysis (MNA) Matrix
///
/// The MNA formulation creates a system of linear equations:
/// [G] [V] = [I]
///
/// Where:
/// - G is the conductance matrix (including voltage source stamps)
/// - V is the vector of unknown voltages and currents
/// - I is the vector of known current sources
#[derive(Clone, Debug)]
pub struct MNAMatrix {
    /// Matrix size (n = num_nodes + num_vsources)
    pub size: usize,
    /// Number of circuit nodes (excluding ground)
    pub num_nodes: usize,
    /// Number of voltage sources
    pub num_vsources: usize,
    /// Matrix entries [size x size]
    pub matrix: Vec<Vec<f64>>,
    /// Right-hand side vector [size]
    pub rhs: Vec<f64>,
}

impl MNAMatrix {
    /// Create a new MNA matrix
    pub fn new(num_nodes: usize, num_vsources: usize) -> Self {
        let size = num_nodes + num_vsources;
        let matrix = vec![vec![0.0; size]; size];
        let rhs = vec![0.0; size];

        Self {
            size,
            num_nodes,
            num_vsources,
            matrix,
            rhs,
        }
    }

    /// Stamp a resistor: G = 1/R
    ///
    /// Updates:
    /// G[n1][n1] += G    G[n1][n2] -= G
    /// G[n2][n1] -= G    G[n2][n2] += G
    pub fn stamp_resistor(&mut self, node_p: usize, node_n: usize, resistance: f64) {
        if resistance == 0.0 {
            return;  // Skip zero resistance (short circuit handled separately)
        }

        let g = 1.0 / resistance;

        if node_p > 0 {
            self.matrix[node_p - 1][node_p - 1] += g;
            if node_n > 0 {
                self.matrix[node_p - 1][node_n - 1] -= g;
            }
        }

        if node_n > 0 {
            self.matrix[node_n - 1][node_n - 1] += g;
            if node_p > 0 {
                self.matrix[node_n - 1][node_p - 1] -= g;
            }
        }
    }

    /// Stamp a conductance (G-element)
    pub fn stamp_conductance(&mut self, node_p: usize, node_n: usize, conductance: f64) {
        if node_p > 0 {
            self.matrix[node_p - 1][node_p - 1] += conductance;
            if node_n > 0 {
                self.matrix[node_p - 1][node_n - 1] -= conductance;
            }
        }

        if node_n > 0 {
            self.matrix[node_n - 1][node_n - 1] += conductance;
            if node_p > 0 {
                self.matrix[node_n - 1][node_p - 1] -= conductance;
            }
        }
    }

    /// Stamp a voltage source
    ///
    /// Adds an auxiliary variable for the current through the voltage source.
    /// vs_idx is the index of the voltage source (0-indexed)
    ///
    /// Updates:
    /// G[n1][n+vs] += 1    G[n+vs][n1] += 1
    /// G[n2][n+vs] -= 1    G[n+vs][n2] -= 1
    /// I[n+vs] = V
    pub fn stamp_voltage_source(
        &mut self,
        node_p: usize,
        node_n: usize,
        vs_idx: usize,
        voltage: f64,
    ) {
        let vs_row = self.num_nodes + vs_idx;

        if node_p > 0 {
            self.matrix[node_p - 1][vs_row] += 1.0;
            self.matrix[vs_row][node_p - 1] += 1.0;
        }

        if node_n > 0 {
            self.matrix[node_n - 1][vs_row] -= 1.0;
            self.matrix[vs_row][node_n - 1] -= 1.0;
        }

        self.rhs[vs_row] = voltage;
    }

    /// Stamp a current source
    ///
    /// Current flows from node_p to node_n
    /// Updates: I[n1] -= I, I[n2] += I
    pub fn stamp_current_source(&mut self, node_p: usize, node_n: usize, current: f64) {
        if node_p > 0 {
            self.rhs[node_p - 1] -= current;
        }
        if node_n > 0 {
            self.rhs[node_n - 1] += current;
        }
    }

    /// Stamp VCVS (Voltage-Controlled Voltage Source)
    ///
    /// Vout = gain * (Vctrl_p - Vctrl_n)
    pub fn stamp_vcvs(
        &mut self,
        node_out_p: usize,
        node_out_n: usize,
        node_ctrl_p: usize,
        node_ctrl_n: usize,
        vs_idx: usize,
        gain: f64,
    ) {
        let vs_row = self.num_nodes + vs_idx;

        // Output connections (like voltage source)
        if node_out_p > 0 {
            self.matrix[node_out_p - 1][vs_row] += 1.0;
            self.matrix[vs_row][node_out_p - 1] += 1.0;
        }
        if node_out_n > 0 {
            self.matrix[node_out_n - 1][vs_row] -= 1.0;
            self.matrix[vs_row][node_out_n - 1] -= 1.0;
        }

        // Control voltage dependency
        if node_ctrl_p > 0 {
            self.matrix[vs_row][node_ctrl_p - 1] -= gain;
        }
        if node_ctrl_n > 0 {
            self.matrix[vs_row][node_ctrl_n - 1] += gain;
        }
    }

    /// Stamp VCCS (Voltage-Controlled Current Source)
    ///
    /// Iout = gm * (Vctrl_p - Vctrl_n)
    pub fn stamp_vccs(
        &mut self,
        node_out_p: usize,
        node_out_n: usize,
        node_ctrl_p: usize,
        node_ctrl_n: usize,
        transconductance: f64,
    ) {
        // VCCS is like a transconductance between control and output nodes
        if node_out_p > 0 && node_ctrl_p > 0 {
            self.matrix[node_out_p - 1][node_ctrl_p - 1] += transconductance;
        }
        if node_out_p > 0 && node_ctrl_n > 0 {
            self.matrix[node_out_p - 1][node_ctrl_n - 1] -= transconductance;
        }
        if node_out_n > 0 && node_ctrl_p > 0 {
            self.matrix[node_out_n - 1][node_ctrl_p - 1] -= transconductance;
        }
        if node_out_n > 0 && node_ctrl_n > 0 {
            self.matrix[node_out_n - 1][node_ctrl_n - 1] += transconductance;
        }
    }

    /// Solve the system using LU decomposition with partial pivoting
    ///
    /// Returns the solution vector [V1, V2, ..., Vn, I_vs1, I_vs2, ...]
    pub fn solve(&self) -> Result<Vec<f64>, String> {
        // Create copies for decomposition
        let mut a = self.matrix.clone();
        let b = self.rhs.clone();
        let n = self.size;

        // LU decomposition with partial pivoting
        let mut pivot = vec![0usize; n];
        for i in 0..n {
            pivot[i] = i;
        }

        for k in 0..n {
            // Find pivot
            let mut max_val = a[k][k].abs();
            let mut max_row = k;

            for i in (k + 1)..n {
                let val = a[i][k].abs();
                if val > max_val {
                    max_val = val;
                    max_row = i;
                }
            }

            if max_val < 1e-12 {
                return Err(format!("Matrix is singular at row {}", k));
            }

            // Swap rows in matrix and pivot array
            if max_row != k {
                a.swap(k, max_row);
                pivot.swap(k, max_row);
            }

            // Elimination
            for i in (k + 1)..n {
                let factor = a[i][k] / a[k][k];
                a[i][k] = factor;

                for j in (k + 1)..n {
                    a[i][j] -= factor * a[k][j];
                }
            }
        }

        // Apply pivoting to RHS
        let mut b_pivot = vec![0.0; n];
        for i in 0..n {
            b_pivot[i] = b[pivot[i]];
        }

        // Forward substitution (Ly = b)
        let mut y = vec![0.0; n];
        for i in 0..n {
            let mut sum = b_pivot[i];
            for j in 0..i {
                sum -= a[i][j] * y[j];
            }
            y[i] = sum;
        }

        // Back substitution (Ux = y)
        let mut x = vec![0.0; n];
        for i in (0..n).rev() {
            let mut sum = y[i];
            for j in (i + 1)..n {
                sum -= a[i][j] * x[j];
            }
            x[i] = sum / a[i][i];
        }

        Ok(x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resistor_stamp() {
        let mut matrix = MNAMatrix::new(2, 0);
        matrix.stamp_resistor(1, 2, 1000.0);

        // G = 1/1000 = 0.001
        assert!((matrix.matrix[0][0] - 0.001).abs() < 1e-10);
        assert!((matrix.matrix[0][1] + 0.001).abs() < 1e-10);
        assert!((matrix.matrix[1][0] + 0.001).abs() < 1e-10);
        assert!((matrix.matrix[1][1] - 0.001).abs() < 1e-10);
    }

    #[test]
    fn test_voltage_source_stamp() {
        let mut matrix = MNAMatrix::new(1, 1);
        matrix.stamp_voltage_source(1, 0, 0, 5.0);

        // Check stamps
        assert!((matrix.matrix[0][1] - 1.0).abs() < 1e-10);
        assert!((matrix.matrix[1][0] - 1.0).abs() < 1e-10);
        assert!((matrix.rhs[1] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_simple_circuit_solve() {
        // Circuit: V1 (5V) -> R1 (1k) -> GND
        // Expected: I = 5V / 1kΩ = 5mA

        let mut matrix = MNAMatrix::new(1, 1);
        matrix.stamp_voltage_source(1, 0, 0, 5.0);
        matrix.stamp_resistor(1, 0, 1000.0);

        let solution = matrix.solve().unwrap();

        // V1 = 5V, I_vs = -0.005A (negative by MNA sign convention)
        assert!((solution[0] - 5.0).abs() < 1e-6);
        assert!((solution[1] + 0.005).abs() < 1e-6);
    }

    #[test]
    fn test_voltage_divider() {
        // Circuit: V1 (10V) -> R1 (1k) -> node 1 -> R2 (1k) -> GND
        // Expected: V_node1 = 5V

        let mut matrix = MNAMatrix::new(1, 1);
        matrix.stamp_voltage_source(2, 0, 0, 10.0);  // V source at node 2
        matrix.stamp_resistor(2, 1, 1000.0);  // R1
        matrix.stamp_resistor(1, 0, 1000.0);  // R2

        // Wait, this needs 2 nodes (node 1 and node 2)
        let mut matrix = MNAMatrix::new(2, 1);
        matrix.stamp_voltage_source(2, 0, 0, 10.0);
        matrix.stamp_resistor(2, 1, 1000.0);
        matrix.stamp_resistor(1, 0, 1000.0);

        let solution = matrix.solve().unwrap();

        // Node 1 should be 5V (voltage divider)
        assert!((solution[0] - 5.0).abs() < 1e-6);
        // Node 2 should be 10V (voltage source)
        assert!((solution[1] - 10.0).abs() < 1e-6);
    }
}
