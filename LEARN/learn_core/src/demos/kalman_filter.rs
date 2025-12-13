//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: kalman_filter.rs | LEARN/learn_core/src/demos/kalman_filter.rs
//! PURPOSE: Kalman Filter demo for sensor fusion (odometry + GPS)
//! MODIFIED: 2025-12-12
//! LAYER: LEARN → learn_core → demos
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! # Kalman Filter Algorithm
//!
//! The Kalman Filter is the optimal estimator for LINEAR systems with GAUSSIAN noise.
//! It maintains a belief as (mean μ, covariance Σ).
//!
//! ## Algorithm Steps:
//!
//! 1. **PREDICT**: Project state forward using motion model
//!    - μ' = A * μ + B * u       (predicted mean)
//!    - Σ' = A * Σ * A^T + Q     (predicted covariance - GROWS)
//!
//! 2. **UPDATE**: Incorporate measurement
//!    - K = Σ' * H^T * (H * Σ' * H^T + R)^(-1)   (Kalman gain)
//!    - μ = μ' + K * (z - H * μ')                 (corrected mean)
//!    - Σ = (I - K * H) * Σ'                      (corrected covariance - SHRINKS)
//!
//! The Kalman Gain K determines how much to trust the measurement vs the prediction.
//! - If R is large (noisy sensor), K is small → trust prediction more
//! - If Σ' is large (uncertain prediction), K is large → trust measurement more

use crate::{Demo, ParamMeta, Rng, Vec2};

/// 2x2 matrix for covariance
#[derive(Clone, Copy, Debug)]
pub struct Mat2 {
    pub m00: f32, pub m01: f32,
    pub m10: f32, pub m11: f32,
}

impl Mat2 {
    pub fn identity() -> Self {
        Self { m00: 1.0, m01: 0.0, m10: 0.0, m11: 1.0 }
    }

    pub fn diag(d0: f32, d1: f32) -> Self {
        Self { m00: d0, m01: 0.0, m10: 0.0, m11: d1 }
    }

    pub fn scale(self, s: f32) -> Self {
        Self {
            m00: self.m00 * s, m01: self.m01 * s,
            m10: self.m10 * s, m11: self.m11 * s,
        }
    }

    pub fn add(self, other: Self) -> Self {
        Self {
            m00: self.m00 + other.m00, m01: self.m01 + other.m01,
            m10: self.m10 + other.m10, m11: self.m11 + other.m11,
        }
    }

    pub fn mul_vec(self, v: Vec2) -> Vec2 {
        Vec2::new(
            self.m00 * v.x + self.m01 * v.y,
            self.m10 * v.x + self.m11 * v.y,
        )
    }

    pub fn mul_mat(self, other: Self) -> Self {
        Self {
            m00: self.m00 * other.m00 + self.m01 * other.m10,
            m01: self.m00 * other.m01 + self.m01 * other.m11,
            m10: self.m10 * other.m00 + self.m11 * other.m10,
            m11: self.m10 * other.m01 + self.m11 * other.m11,
        }
    }

    pub fn transpose(self) -> Self {
        Self {
            m00: self.m00, m01: self.m10,
            m10: self.m01, m11: self.m11,
        }
    }

    /// Inverse of 2x2 matrix
    pub fn inverse(self) -> Option<Self> {
        let det = self.m00 * self.m11 - self.m01 * self.m10;
        if det.abs() < 1e-10 {
            return None;
        }
        Some(Self {
            m00: self.m11 / det,  m01: -self.m01 / det,
            m10: -self.m10 / det, m11: self.m00 / det,
        })
    }

    /// Get eigenvalues for ellipse visualization
    pub fn eigenvalues(&self) -> (f32, f32) {
        let trace = self.m00 + self.m11;
        let det = self.m00 * self.m11 - self.m01 * self.m10;
        let disc = (trace * trace / 4.0 - det).max(0.0).sqrt();
        (trace / 2.0 + disc, (trace / 2.0 - disc).max(0.001))
    }

    /// Get angle of principal axis (for ellipse orientation)
    pub fn principal_angle(&self) -> f32 {
        if self.m01.abs() < 1e-10 {
            0.0
        } else {
            let (l1, _) = self.eigenvalues();
            (self.m01 / (l1 - self.m11)).atan()
        }
    }
}

/// Current phase of Kalman filter (for visualization)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KFPhase {
    Predict,
    Update,
}

impl KFPhase {
    pub fn name(&self) -> &'static str {
        match self {
            KFPhase::Predict => "PREDICT",
            KFPhase::Update => "UPDATE",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            KFPhase::Predict => "Projecting state forward with motion model. Uncertainty GROWS.",
            KFPhase::Update => "Incorporating GPS measurement. Uncertainty SHRINKS.",
        }
    }
}

/// Kalman Filter demo for 2D robot tracking with GPS fusion
#[derive(Clone)]
pub struct KalmanFilterDemo {
    // True robot state (hidden, for visualization)
    pub true_pos: Vec2,
    pub true_vel: Vec2,
    pub true_path: Vec<Vec2>,

    // Kalman filter state
    pub kf_pos: Vec2,      // estimated position (mean)
    pub kf_cov: Mat2,      // position covariance
    pub kf_path: Vec<Vec2>, // estimated trajectory

    // Last GPS measurement (for visualization)
    pub last_gps: Option<Vec2>,
    pub gps_history: Vec<Vec2>,

    // Kalman gain (for visualization)
    pub kalman_gain: Mat2,

    // Noise parameters
    pub process_noise: f32,    // Q: motion model uncertainty
    pub measurement_noise: f32, // R: GPS measurement uncertainty
    pub gps_interval: usize,   // frames between GPS updates

    // Algorithm state
    pub phase: KFPhase,
    frame_count: usize,

    // Time and motion
    time: f32,

    // RNG
    rng: Rng,
}

impl Default for KalmanFilterDemo {
    fn default() -> Self {
        Self {
            true_pos: Vec2::new(0.5, 0.5),
            true_vel: Vec2::new(0.1, 0.05),
            true_path: Vec::new(),
            kf_pos: Vec2::new(0.5, 0.5),
            kf_cov: Mat2::diag(0.01, 0.01),
            kf_path: Vec::new(),
            last_gps: None,
            gps_history: Vec::new(),
            kalman_gain: Mat2::identity(),
            process_noise: 0.01,      // BEST: minimum process noise
            measurement_noise: 0.1,   // BEST: minimum GPS noise
            gps_interval: 1,          // BEST: GPS every frame
            phase: KFPhase::Predict,
            frame_count: 0,
            time: 0.0,
            rng: Rng::new(42),
        }
    }
}

impl KalmanFilterDemo {
    /// Sample from Gaussian distribution
    fn gaussian(&mut self, mean: f32, std_dev: f32) -> f32 {
        let u1 = self.rng.range(0.0001, 1.0);
        let u2 = self.rng.range(0.0, 1.0);
        let z = (-2.0 * u1.ln()).sqrt() * (std::f32::consts::TAU * u2).cos();
        mean + std_dev * z
    }

    /// Move the true robot (hidden state)
    fn move_robot(&mut self, dt: f32) {
        // Slowly varying velocity (makes it interesting)
        self.time += dt;
        let omega = 0.5; // angular frequency for velocity changes
        self.true_vel = Vec2::new(
            0.15 * (self.time * omega).cos(),
            0.1 * (self.time * omega * 1.3).sin(),
        );

        // Move robot
        self.true_pos.x += self.true_vel.x * dt;
        self.true_pos.y += self.true_vel.y * dt;

        // Bounce off walls
        if self.true_pos.x < 0.1 || self.true_pos.x > 0.9 {
            self.true_vel.x = -self.true_vel.x;
            self.true_pos.x = self.true_pos.x.clamp(0.1, 0.9);
        }
        if self.true_pos.y < 0.1 || self.true_pos.y > 0.9 {
            self.true_vel.y = -self.true_vel.y;
            self.true_pos.y = self.true_pos.y.clamp(0.1, 0.9);
        }

        // Record true path
        self.true_path.push(self.true_pos);
        if self.true_path.len() > 300 {
            self.true_path.remove(0);
        }
    }

    /// PREDICT step: project state forward with motion model
    fn predict(&mut self, dt: f32) {
        // Motion model: x' = x + v*dt + noise
        // For simplicity, we assume velocity is roughly estimated from odometry
        let estimated_vel = Vec2::new(
            self.true_vel.x + self.gaussian(0.0, self.process_noise * 0.5),
            self.true_vel.y + self.gaussian(0.0, self.process_noise * 0.5),
        );

        // Predict mean: μ' = A * μ + B * u (A = I, B*u = estimated_vel * dt)
        self.kf_pos.x += estimated_vel.x * dt;
        self.kf_pos.y += estimated_vel.y * dt;

        // Predict covariance: Σ' = A * Σ * A^T + Q
        // A = I for position-only model, so Σ' = Σ + Q
        let q = Mat2::diag(self.process_noise * dt, self.process_noise * dt);
        self.kf_cov = self.kf_cov.add(q);

        // Keep in bounds
        self.kf_pos.x = self.kf_pos.x.clamp(0.05, 0.95);
        self.kf_pos.y = self.kf_pos.y.clamp(0.05, 0.95);

        self.phase = KFPhase::Predict;
    }

    /// UPDATE step: incorporate GPS measurement
    fn update(&mut self) {
        // Simulate GPS measurement (true position + noise)
        let gps_measurement = Vec2::new(
            self.true_pos.x + self.gaussian(0.0, self.measurement_noise * 0.1),
            self.true_pos.y + self.gaussian(0.0, self.measurement_noise * 0.1),
        );

        self.last_gps = Some(gps_measurement);
        self.gps_history.push(gps_measurement);
        if self.gps_history.len() > 50 {
            self.gps_history.remove(0);
        }

        // H = I (we directly measure position)
        // R = measurement noise covariance
        let r = Mat2::diag(
            self.measurement_noise * 0.1,
            self.measurement_noise * 0.1,
        );

        // Kalman gain: K = Σ' * H^T * (H * Σ' * H^T + R)^(-1)
        // With H = I: K = Σ' * (Σ' + R)^(-1)
        let s = self.kf_cov.add(r); // innovation covariance
        if let Some(s_inv) = s.inverse() {
            self.kalman_gain = self.kf_cov.mul_mat(s_inv);

            // Update mean: μ = μ' + K * (z - H * μ')
            let innovation = Vec2::new(
                gps_measurement.x - self.kf_pos.x,
                gps_measurement.y - self.kf_pos.y,
            );
            let correction = self.kalman_gain.mul_vec(innovation);
            self.kf_pos.x += correction.x;
            self.kf_pos.y += correction.y;

            // Update covariance: Σ = (I - K * H) * Σ'
            // With H = I: Σ = (I - K) * Σ'
            let i_minus_k = Mat2 {
                m00: 1.0 - self.kalman_gain.m00,
                m01: -self.kalman_gain.m01,
                m10: -self.kalman_gain.m10,
                m11: 1.0 - self.kalman_gain.m11,
            };
            self.kf_cov = i_minus_k.mul_mat(self.kf_cov);

            // Ensure covariance stays positive definite
            self.kf_cov.m00 = self.kf_cov.m00.max(0.0001);
            self.kf_cov.m11 = self.kf_cov.m11.max(0.0001);
        }

        self.phase = KFPhase::Update;
    }

    /// Get error between estimate and true position
    pub fn error(&self) -> f32 {
        self.true_pos.distance(self.kf_pos)
    }

    /// Get uncertainty (trace of covariance)
    pub fn uncertainty(&self) -> f32 {
        (self.kf_cov.m00 + self.kf_cov.m11).sqrt()
    }

    /// Get ellipse parameters for visualization (semi-axes and angle)
    pub fn covariance_ellipse(&self) -> (f32, f32, f32) {
        let (l1, l2) = self.kf_cov.eigenvalues();
        let angle = self.kf_cov.principal_angle();
        // 2-sigma ellipse (95% confidence)
        (2.0 * l1.sqrt(), 2.0 * l2.sqrt(), angle)
    }
}

impl Demo for KalmanFilterDemo {
    fn reset(&mut self, seed: u64) {
        self.rng = Rng::new(seed);
        self.time = 0.0;
        self.frame_count = 0;

        // Reset true state
        self.true_pos = Vec2::new(
            self.rng.range(0.3, 0.7),
            self.rng.range(0.3, 0.7),
        );
        self.true_vel = Vec2::new(0.1, 0.05);
        self.true_path.clear();

        // Initialize KF state with some uncertainty
        self.kf_pos = Vec2::new(
            self.true_pos.x + self.gaussian(0.0, 0.1),
            self.true_pos.y + self.gaussian(0.0, 0.1),
        );
        self.kf_cov = Mat2::diag(0.02, 0.02);
        self.kf_path.clear();

        self.last_gps = None;
        self.gps_history.clear();
        self.kalman_gain = Mat2::identity();
        self.phase = KFPhase::Predict;
    }

    fn step(&mut self, dt: f32) {
        self.frame_count += 1;

        // Move true robot
        self.move_robot(dt);

        // Always predict (odometry)
        self.predict(dt);

        // Update only when GPS is available
        if self.frame_count % self.gps_interval == 0 {
            self.update();
        }

        // Record estimated path
        self.kf_path.push(self.kf_pos);
        if self.kf_path.len() > 300 {
            self.kf_path.remove(0);
        }
    }

    fn set_param(&mut self, name: &str, value: f32) -> bool {
        match name {
            "process_noise" => {
                self.process_noise = value.clamp(0.01, 1.0);
                true
            }
            "measurement_noise" => {
                self.measurement_noise = value.clamp(0.1, 2.0);
                true
            }
            "gps_interval" => {
                self.gps_interval = (value as usize).clamp(1, 50);
                true
            }
            _ => false,
        }
    }

    fn params() -> &'static [ParamMeta] {
        &[
            ParamMeta {
                name: "process_noise",
                label: "Process Noise (Q)",
                min: 0.01,
                max: 1.0,
                step: 0.01,
                default: 0.01,   // BEST: minimum noise
            },
            ParamMeta {
                name: "measurement_noise",
                label: "Measurement Noise (R)",
                min: 0.1,
                max: 2.0,
                step: 0.1,
                default: 0.1,    // BEST: minimum noise
            },
            ParamMeta {
                name: "gps_interval",
                label: "GPS Interval",
                min: 1.0,
                max: 50.0,
                step: 1.0,
                default: 1.0,    // BEST: GPS every frame
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reset_initializes() {
        let mut demo = KalmanFilterDemo::default();
        demo.reset(42);
        assert!(demo.true_pos.x > 0.0 && demo.true_pos.x < 1.0);
        assert!(demo.kf_cov.m00 > 0.0);
    }

    #[test]
    fn test_predict_increases_uncertainty() {
        let mut demo = KalmanFilterDemo::default();
        demo.reset(42);
        let cov_before = demo.kf_cov.m00;
        demo.predict(0.1);
        assert!(demo.kf_cov.m00 > cov_before, "Covariance should grow during prediction");
    }

    #[test]
    fn test_update_decreases_uncertainty() {
        let mut demo = KalmanFilterDemo::default();
        demo.reset(42);
        // Run a few predictions to build up uncertainty
        for _ in 0..10 {
            demo.predict(0.1);
        }
        let cov_before = demo.kf_cov.m00;
        demo.update();
        assert!(demo.kf_cov.m00 < cov_before, "Covariance should shrink after update");
    }

    #[test]
    fn test_deterministic() {
        let mut demo1 = KalmanFilterDemo::default();
        let mut demo2 = KalmanFilterDemo::default();

        demo1.reset(123);
        demo2.reset(123);

        for _ in 0..20 {
            demo1.step(0.016);
            demo2.step(0.016);
        }

        assert!(
            (demo1.kf_pos.x - demo2.kf_pos.x).abs() < 1e-6,
            "Should be deterministic"
        );
    }
}
