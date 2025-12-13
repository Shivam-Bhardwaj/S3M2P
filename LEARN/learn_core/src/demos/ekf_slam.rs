//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: ekf_slam.rs | LEARN/learn_core/src/demos/ekf_slam.rs
//! PURPOSE: EKF SLAM demo - robot localization AND mapping simultaneously
//! MODIFIED: 2025-12-12
//! LAYER: LEARN → learn_core → demos
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! # EKF SLAM
//!
//! The chicken-and-egg problem: you need a map to localize, but need to know
//! where you are to build a map. EKF SLAM estimates both simultaneously.
//!
//! Key insight: Robot position and landmark positions are CORRELATED.
//! When we get better info about one, it helps the others through the
//! covariance matrix.

use crate::{Demo, ParamMeta, Rng, Vec2};

/// A landmark in the map
#[derive(Clone, Debug)]
pub struct SlamLandmark {
    /// Estimated position
    pub pos: Vec2,
    /// Uncertainty (variance in x and y)
    pub variance: Vec2,
    /// Number of times observed
    pub observations: u32,
}

/// EKF SLAM Demo
#[derive(Clone)]
pub struct EkfSlamDemo {
    // True robot state (hidden from filter)
    pub true_pos: Vec2,
    pub true_theta: f32,

    // Estimated robot state
    pub est_pos: Vec2,
    pub est_theta: f32,
    pub robot_variance: Vec2,

    // Discovered landmarks
    pub landmarks: Vec<SlamLandmark>,

    // True landmark positions (for simulation)
    true_landmarks: Vec<Vec2>,

    // Robot path history
    pub robot_path: Vec<Vec2>,
    pub est_path: Vec<Vec2>,

    // Parameters
    pub motion_noise: f32,
    pub sensor_noise: f32,
    pub sensor_range: f32,

    // Simulation state
    time: f32,
    rng: Rng,
    max_history: usize,

    // Last observation info (for visualization)
    pub last_observed_idx: Option<usize>,
    pub last_was_new: bool,
}

impl Default for EkfSlamDemo {
    fn default() -> Self {
        // Create true landmark positions in a scattered pattern
        let true_landmarks = vec![
            Vec2::new(0.2, 0.2),
            Vec2::new(0.8, 0.2),
            Vec2::new(0.8, 0.8),
            Vec2::new(0.2, 0.8),
            Vec2::new(0.5, 0.5),
            Vec2::new(0.35, 0.5),
            Vec2::new(0.65, 0.5),
        ];

        Self {
            true_pos: Vec2::new(0.5, 0.3),
            true_theta: 0.0,
            est_pos: Vec2::new(0.5, 0.3),
            est_theta: 0.0,
            robot_variance: Vec2::new(0.001, 0.001),
            landmarks: Vec::new(),
            true_landmarks,
            robot_path: Vec::new(),
            est_path: Vec::new(),
            motion_noise: 0.005,     // BEST: minimum noise
            sensor_noise: 0.01,      // BEST: minimum sensor noise
            sensor_range: 0.6,       // BEST: maximum range
            time: 0.0,
            rng: Rng::new(42),
            max_history: 200,
            last_observed_idx: None,
            last_was_new: false,
        }
    }
}

impl EkfSlamDemo {
    fn gaussian(&mut self, std_dev: f32) -> f32 {
        let u1 = self.rng.range(0.0001, 1.0);
        let u2 = self.rng.range(0.0, 1.0);
        let z = (-2.0 * u1.ln()).sqrt() * (std::f32::consts::TAU * u2).cos();
        std_dev * z
    }

    /// Move the robot
    fn move_robot(&mut self, dt: f32) {
        // Circular motion for demo
        self.time += dt;
        let angular_vel = 0.3;
        let linear_vel = 0.08;

        // True motion
        self.true_theta += angular_vel * dt;
        let dx = linear_vel * dt * self.true_theta.cos();
        let dy = linear_vel * dt * self.true_theta.sin();

        self.true_pos.x = (self.true_pos.x + dx).clamp(0.05, 0.95);
        self.true_pos.y = (self.true_pos.y + dy).clamp(0.05, 0.95);

        // Estimated motion (with noise)
        let noisy_dx = dx + self.gaussian(self.motion_noise * dt);
        let noisy_dy = dy + self.gaussian(self.motion_noise * dt);

        self.est_pos.x = (self.est_pos.x + noisy_dx).clamp(0.05, 0.95);
        self.est_pos.y = (self.est_pos.y + noisy_dy).clamp(0.05, 0.95);
        self.est_theta += angular_vel * dt + self.gaussian(0.05 * dt);

        // Increase uncertainty due to motion
        self.robot_variance.x += self.motion_noise * self.motion_noise * dt;
        self.robot_variance.y += self.motion_noise * self.motion_noise * dt;

        // Record paths
        self.robot_path.push(self.true_pos);
        self.est_path.push(self.est_pos);
        if self.robot_path.len() > self.max_history {
            self.robot_path.remove(0);
            self.est_path.remove(0);
        }
    }

    /// Observe landmarks and update
    fn observe_landmarks(&mut self) {
        self.last_observed_idx = None;
        self.last_was_new = false;

        // Clone data to avoid borrow issues
        let true_landmarks = self.true_landmarks.clone();
        let true_pos = self.true_pos;
        let sensor_range = self.sensor_range;
        let sensor_noise = self.sensor_noise;

        // Collect observations to process
        let mut updates: Vec<(usize, Vec2)> = Vec::new();
        let mut additions: Vec<Vec2> = Vec::new();

        // First pass: find landmarks in range
        for true_lm in &true_landmarks {
            // Check if landmark is in range
            let dx = true_lm.x - true_pos.x;
            let dy = true_lm.y - true_pos.y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist > sensor_range {
                continue;
            }

            // Generate noise
            let noise_x = self.gaussian(sensor_noise);
            let noise_y = self.gaussian(sensor_noise);
            let noisy_x = true_lm.x + noise_x;
            let noisy_y = true_lm.y + noise_y;
            let measured_pos = Vec2::new(noisy_x, noisy_y);

            // Find matching landmark in our map
            let mut found_idx = None;
            for (i, lm) in self.landmarks.iter().enumerate() {
                let lm_dx = lm.pos.x - noisy_x;
                let lm_dy = lm.pos.y - noisy_y;
                let lm_dist = (lm_dx * lm_dx + lm_dy * lm_dy).sqrt();
                if lm_dist < 0.1 {
                    found_idx = Some(i);
                    break;
                }
            }

            if let Some(idx) = found_idx {
                updates.push((idx, measured_pos));
            } else {
                additions.push(measured_pos);
            }
        }

        // Second pass: apply updates
        for (idx, measured_pos) in updates {
            self.update_landmark(idx, measured_pos);
            self.last_observed_idx = Some(idx);
            self.last_was_new = false;
        }

        // Third pass: add new landmarks
        for measured_pos in additions {
            self.add_landmark(measured_pos);
            self.last_observed_idx = Some(self.landmarks.len() - 1);
            self.last_was_new = true;
        }
    }

    fn add_landmark(&mut self, pos: Vec2) {
        // New landmark - initialize with high uncertainty
        self.landmarks.push(SlamLandmark {
            pos,
            variance: Vec2::new(
                self.robot_variance.x + self.sensor_noise * self.sensor_noise,
                self.robot_variance.y + self.sensor_noise * self.sensor_noise,
            ),
            observations: 1,
        });
    }

    fn update_landmark(&mut self, idx: usize, measurement: Vec2) {
        let lm = &mut self.landmarks[idx];
        lm.observations += 1;

        // Simple Kalman-style update
        let measurement_var = self.sensor_noise * self.sensor_noise;

        // Kalman gain for x
        let k_x = lm.variance.x / (lm.variance.x + measurement_var);
        let k_y = lm.variance.y / (lm.variance.y + measurement_var);

        // Update position
        lm.pos.x += k_x * (measurement.x - lm.pos.x);
        lm.pos.y += k_y * (measurement.y - lm.pos.y);

        // Update variance (reduce uncertainty)
        lm.variance.x *= 1.0 - k_x;
        lm.variance.y *= 1.0 - k_y;

        // Also reduce robot uncertainty (correlation effect)
        self.robot_variance.x *= 0.95;
        self.robot_variance.y *= 0.95;

        // Correct robot position towards landmark
        // This simulates the SLAM correlation effect
        if lm.observations > 1 {
            let robot_k = 0.1; // Small correction
            self.est_pos.x += robot_k * (measurement.x - lm.pos.x);
            self.est_pos.y += robot_k * (measurement.y - lm.pos.y);
        }
    }

    /// Get error between estimate and true position
    pub fn robot_error(&self) -> f32 {
        let dx = self.est_pos.x - self.true_pos.x;
        let dy = self.est_pos.y - self.true_pos.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Get average landmark error
    pub fn map_error(&self) -> f32 {
        if self.landmarks.is_empty() {
            return 0.0;
        }

        let mut total_error = 0.0;
        let mut count = 0;

        for lm in &self.landmarks {
            // Find closest true landmark
            let mut min_dist = f32::MAX;
            for true_lm in &self.true_landmarks {
                let dx = lm.pos.x - true_lm.x;
                let dy = lm.pos.y - true_lm.y;
                let dist = (dx * dx + dy * dy).sqrt();
                min_dist = min_dist.min(dist);
            }
            total_error += min_dist;
            count += 1;
        }

        total_error / count as f32
    }
}

impl Demo for EkfSlamDemo {
    fn reset(&mut self, seed: u64) {
        self.rng = Rng::new(seed);
        self.true_pos = Vec2::new(0.5, 0.3);
        self.true_theta = 0.0;
        self.est_pos = Vec2::new(0.5, 0.3);
        self.est_theta = 0.0;
        self.robot_variance = Vec2::new(0.001, 0.001);
        self.landmarks.clear();
        self.robot_path.clear();
        self.est_path.clear();
        self.time = 0.0;
        self.last_observed_idx = None;
        self.last_was_new = false;
    }

    fn step(&mut self, dt: f32) {
        self.move_robot(dt);
        self.observe_landmarks();
    }

    fn set_param(&mut self, name: &str, value: f32) -> bool {
        match name {
            "motion_noise" => {
                self.motion_noise = value.clamp(0.005, 0.1);
                true
            }
            "sensor_noise" => {
                self.sensor_noise = value.clamp(0.01, 0.1);
                true
            }
            "sensor_range" => {
                self.sensor_range = value.clamp(0.1, 0.5);
                true
            }
            _ => false,
        }
    }

    fn params() -> &'static [ParamMeta] {
        &[
            ParamMeta {
                name: "motion_noise",
                label: "Motion Noise",
                min: 0.005,
                max: 0.1,
                step: 0.005,
                default: 0.005,   // BEST: minimum noise
            },
            ParamMeta {
                name: "sensor_noise",
                label: "Sensor Noise",
                min: 0.01,
                max: 0.1,
                step: 0.01,
                default: 0.01,    // BEST: minimum noise
            },
            ParamMeta {
                name: "sensor_range",
                label: "Sensor Range",
                min: 0.1,
                max: 0.6,
                step: 0.05,
                default: 0.6,     // BEST: maximum range
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reset() {
        let mut demo = EkfSlamDemo::default();
        demo.step(0.1);
        demo.step(0.1);
        demo.reset(123);
        assert!(demo.landmarks.is_empty());
        assert!(demo.robot_path.is_empty());
    }

    #[test]
    fn test_landmark_discovery() {
        let mut demo = EkfSlamDemo::default();
        demo.sensor_range = 0.5; // Large range to discover landmarks

        for _ in 0..100 {
            demo.step(0.016);
        }

        // Should have discovered some landmarks
        assert!(!demo.landmarks.is_empty());
    }

    #[test]
    fn test_deterministic() {
        let mut demo1 = EkfSlamDemo::default();
        let mut demo2 = EkfSlamDemo::default();

        demo1.reset(42);
        demo2.reset(42);

        for _ in 0..50 {
            demo1.step(0.016);
            demo2.step(0.016);
        }

        assert!((demo1.true_pos.x - demo2.true_pos.x).abs() < 1e-6);
    }
}
