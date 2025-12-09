use super::types::TransientResult;
use std::f64::consts::PI;

/// Simulate PLL step response using discrete time domain model
#[allow(clippy::too_many_arguments)]
pub fn simulate_step_response(
    k_phi: f64, // Charge pump current (A/rad)
    k_vco: f64, // VCO gain (Hz/V) -> will convert to rad/s/V
    n_divider: f64,
    r1: f64,
    c1: f64,
    c2: f64,
    start_freq_hz: f64,
    target_freq_hz: f64,
    sim_time_s: f64,
) -> TransientResult {
    // Simulation parameters
    let dt = 1.0 / (target_freq_hz * 0.1).min(100e6); // Time step (oversampling)
    let dt = dt.max(1e-9); // Limit min step to 1ns for performance
    let steps = (sim_time_s / dt) as usize;

    // Pre-allocate vectors
    let mut time = Vec::with_capacity(steps);
    let mut freq = Vec::with_capacity(steps);
    let mut phase_error = Vec::with_capacity(steps);

    // Initial state
    let k_vco_hz = k_vco; // Hz/V
    let v_c1 = start_freq_hz / k_vco_hz;
    let v_c2 = v_c1; // Initially capacitors are equalized

    let mut current_freq = start_freq_hz;
    let mut phase_ref = 0.0;
    let mut phase_div = 0.0;

    let ref_freq = target_freq_hz / n_divider; // Assuming locked to target eventually

    // Loop filter state
    // We use a simple difference equation for the loop filter impedance Z(s)
    // Z(s) = (1 + s*R1*C1) / (s*(C1+C2) * (1 + s*R1*C1*C2/(C1+C2)))
    // This is hard to discretize directly.
    // Better: State space or nodal analysis.
    // I_cp -> Node 1 (C2, R1) -> Node 2 (C1) -> GND
    // V_tune = V_node1
    // I_cp = C2 * d(V_node1 - 0)/dt + (V_node1 - V_node2)/R1
    // (V_node1 - V_node2)/R1 = C1 * dV_node2/dt

    // Discretized (Forward Euler):
    // dV2 = (V1 - V2)/(R1*C1) * dt
    // dV1 = (I_cp - (V1-V2)/R1) / C2 * dt

    let mut v1 = v_c2;
    let mut v2 = v_c1;

    let mut lock_time = 0.0;
    let mut locked = false;
    let lock_threshold = 1e-3; // 1 kHz error
    let mut max_freq = start_freq_hz;

    // Downsample output to avoid huge arrays
    let downsample = 100;

    for i in 0..steps {
        let t = i as f64 * dt;

        // 1. Phase Detector
        // Integrate frequencies to get phase
        phase_ref += ref_freq * dt;
        phase_div += (current_freq / n_divider) * dt;

        // Wrap phases? Not strictly necessary for linear model, but good for PFD
        // Linear model: Phase Error = Phase_Ref - Phase_Div
        let phi_e = (phase_ref - phase_div) * 2.0 * PI; // Radians

        // 2. Charge Pump
        // let i_cp = k_phi * phi_e / (2.0 * PI); // Average current
        // Note: k_phi is usually Amps/rad or Amps/cycle?
        // Standard definition: I_pump * phi_e / (2*pi) is the average current.
        // If k_phi is passed as I_pump (Amps), then we use phi_e / 2pi.
        // If k_phi is Amps/rad, then just k_phi * phi_e.
        // In mod.rs: k_phi = charge_pump_current_ua * 1e-6. This is I_pump.
        // So average current = I_pump * phi_e / (2*pi).
        // Wait, standard linear model Gain K_phi = I_cp / (2*pi).
        // Let's assume the input k_phi IS the gain I_cp/(2*pi) or just I_cp?
        // Checking mod.rs: "let k_phi = charge_pump_current_ua * 1e-6;" -> This is I_pump.
        // So Gain is I_pump / (2*pi).

        let i_avg = k_phi * phi_e / (2.0 * PI);

        // 3. Loop Filter (State update)
        let v1_old = v1;
        let v2_old = v2;

        // dV2 = (V1 - V2)/(R1*C1) * dt
        let dv2 = (v1_old - v2_old) / (r1 * c1) * dt;
        v2 += dv2;

        // dV1 = (I_avg - (V1-V2)/R1) / C2 * dt
        let dv1 = (i_avg - (v1_old - v2_old) / r1) / c2 * dt;
        v1 += dv1;

        // 4. VCO
        current_freq = v1 * k_vco_hz; // Simple linear VCO

        // Track metrics
        if current_freq > max_freq {
            max_freq = current_freq;
        }

        let freq_error = (current_freq - target_freq_hz).abs();
        if !locked && freq_error < target_freq_hz * lock_threshold {
            // Check if it stays locked? simplified for now
            lock_time = t;
            locked = true;
        } else if freq_error > target_freq_hz * lock_threshold {
            locked = false;
        }

        // Store data
        if i % downsample == 0 {
            time.push(t);
            freq.push(current_freq);
            phase_error.push(phi_e * 180.0 / PI);
        }
    }

    let overshoot = if max_freq > target_freq_hz {
        (max_freq - target_freq_hz) / (target_freq_hz - start_freq_hz).abs() * 100.0
    } else {
        0.0
    };

    TransientResult {
        time_s: time,
        freq_hz: freq,
        phase_error_deg: phase_error,
        lock_time_us: lock_time * 1e6,
        overshoot_percent: overshoot,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_response() {
        let result = simulate_step_response(
            1e-3,    // k_phi
            10e6,    // k_vco
            100.0,   // N
            10e3,    // R1
            1e-9,    // C1
            100e-12, // C2
            900e6,   // Start
            1e9,     // Target
            100e-6,  // 100us
        );

        assert!(result.time_s.len() > 0);
        assert!(result.freq_hz.len() > 0);
        // Check if it moves towards target
        let final_freq = *result.freq_hz.last().unwrap();
        assert!((final_freq - 1e9).abs() < 100e6); // Should be somewhat close
    }
}
