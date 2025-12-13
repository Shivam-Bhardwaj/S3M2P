//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: particle_filter.rs | LEARN/learn_core/src/demos/particle_filter.rs
//! PURPOSE: 2D robot localization using particle filter (Monte Carlo Localization)
//! MODIFIED: 2025-12-12
//! LAYER: LEARN → learn_core → demos
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! # Particle Filter Algorithm
//!
//! The particle filter (Monte Carlo Localization) estimates robot position using
//! a set of weighted samples (particles). Each particle is a hypothesis about
//! where the robot might be.
//!
//! ## Algorithm Steps (repeated each timestep):
//!
//! 1. **PREDICT**: Move each particle according to the motion command + noise
//!    - Same command as robot, but with added uncertainty
//!    - Particles spread out due to motion noise
//!
//! 2. **UPDATE**: Weight particles by how well they match sensor readings
//!    - Measure distances from true robot to landmarks
//!    - For each particle, compute likelihood it would see those distances
//!    - Particles near true position get high weights
//!
//! 3. **RESAMPLE**: Replace low-weight particles with copies of high-weight ones
//!    - This focuses computational resources on likely hypotheses
//!    - Uses low-variance resampling for better diversity
//!
//! 4. **ESTIMATE**: Compute weighted average of all particles
//!    - This gives the best guess of robot position

use crate::{Demo, ParamMeta, Rng, Vec2};

/// Current phase of the particle filter algorithm (for visualization)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PFPhase {
    /// Moving particles according to motion model
    Predict,
    /// Computing weights from sensor measurements
    Update,
    /// Replacing low-weight particles with high-weight copies
    Resample,
    /// Computing estimated pose from weighted particles
    Estimate,
}

impl PFPhase {
    pub fn name(&self) -> &'static str {
        match self {
            PFPhase::Predict => "PREDICT",
            PFPhase::Update => "UPDATE",
            PFPhase::Resample => "RESAMPLE",
            PFPhase::Estimate => "ESTIMATE",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            PFPhase::Predict => "Moving particles with motion command + noise",
            PFPhase::Update => "Weighting particles by sensor match quality",
            PFPhase::Resample => "Duplicating high-weight, removing low-weight particles",
            PFPhase::Estimate => "Computing weighted average position",
        }
    }

    pub fn next(&self) -> PFPhase {
        match self {
            PFPhase::Predict => PFPhase::Update,
            PFPhase::Update => PFPhase::Resample,
            PFPhase::Resample => PFPhase::Estimate,
            PFPhase::Estimate => PFPhase::Predict,
        }
    }
}

/// A single particle representing a hypothesis about robot pose
#[derive(Clone, Copy, Debug)]
pub struct Particle {
    pub pos: Vec2,
    pub theta: f32,     // heading angle in radians
    pub weight: f32,    // importance weight (sum to 1.0)
    pub prev_pos: Vec2, // for visualizing motion
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            pos: Vec2::ZERO,
            theta: 0.0,
            weight: 1.0,
            prev_pos: Vec2::ZERO,
        }
    }
}

/// Range measurement to a landmark
#[derive(Clone, Copy, Debug)]
pub struct Measurement {
    pub landmark_idx: usize,
    pub range: f32,
    pub noisy_range: f32, // what the sensor actually measured
}

/// Particle filter demo for 2D robot localization
#[derive(Clone)]
pub struct ParticleFilterDemo {
    // True robot state (hidden from the filter, known for visualization)
    pub true_pos: Vec2,
    pub true_theta: f32,
    pub true_prev_pos: Vec2, // for drawing motion trail

    // Motion command (velocity model: forward speed + turn rate)
    pub cmd_forward: f32, // forward velocity (units/sec)
    pub cmd_turn: f32,    // turn rate (rad/sec)

    // Particles
    pub particles: Vec<Particle>,
    num_particles: usize,

    // Estimated state (weighted mean of particles)
    pub est_pos: Vec2,
    pub est_theta: f32,

    // Fixed landmarks for sensing (known map)
    pub landmarks: Vec<Vec2>,

    // Current sensor measurements
    pub measurements: Vec<Measurement>,

    // Noise parameters
    motion_noise: f32, // std dev of motion noise
    sensor_noise: f32, // std dev of range measurement noise

    // Algorithm state
    pub phase: PFPhase,
    pub step_mode: bool, // if true, user controls each phase manually
    phase_timer: f32,    // for auto-advancing phases in continuous mode

    // Statistics for display
    pub effective_particles: f32, // N_eff = 1 / sum(w^2), measures particle diversity
    pub best_particle_weight: f32,
    pub error_history: Vec<f32>, // track error over time

    // Time tracking
    time: f32,

    // RNG
    rng: Rng,
}

impl Default for ParticleFilterDemo {
    fn default() -> Self {
        Self {
            true_pos: Vec2::new(0.3, 0.3),
            true_theta: 0.0,
            true_prev_pos: Vec2::new(0.3, 0.3),
            cmd_forward: 0.15,
            cmd_turn: 0.5,
            particles: Vec::new(),
            num_particles: 500,       // BEST: maximum particles
            est_pos: Vec2::new(0.5, 0.5),
            est_theta: 0.0,
            landmarks: Vec::new(),
            measurements: Vec::new(),
            motion_noise: 0.0,        // BEST: no motion noise
            sensor_noise: 0.01,       // BEST: minimum sensor noise
            phase: PFPhase::Predict,
            step_mode: false,
            phase_timer: 0.0,
            effective_particles: 0.0,
            best_particle_weight: 0.0,
            error_history: Vec::new(),
            time: 0.0,
            rng: Rng::new(42),
        }
    }
}

impl ParticleFilterDemo {
    /// Initialize particles uniformly across the space
    fn init_particles(&mut self) {
        self.particles.clear();
        let uniform_weight = 1.0 / self.num_particles as f32;

        for _ in 0..self.num_particles {
            let pos = Vec2::new(
                self.rng.range(0.1, 0.9),
                self.rng.range(0.1, 0.9),
            );
            self.particles.push(Particle {
                pos,
                theta: self.rng.range(0.0, std::f32::consts::TAU),
                weight: uniform_weight,
                prev_pos: pos,
            });
        }
        self.compute_statistics();
    }

    /// Sample from Gaussian distribution using Box-Muller transform
    fn gaussian(&mut self, mean: f32, std_dev: f32) -> f32 {
        let u1 = self.rng.range(0.0001, 1.0);
        let u2 = self.rng.range(0.0, 1.0);
        let z = (-2.0 * u1.ln()).sqrt() * (std::f32::consts::TAU * u2).cos();
        mean + std_dev * z
    }

    /// Move the true robot according to velocity commands
    fn move_robot(&mut self, dt: f32) {
        self.true_prev_pos = self.true_pos;

        // Differential drive motion model
        // Turn first, then move forward in new direction
        self.true_theta += self.cmd_turn * dt;
        self.true_theta = self.true_theta.rem_euclid(std::f32::consts::TAU);

        self.true_pos.x += self.cmd_forward * self.true_theta.cos() * dt;
        self.true_pos.y += self.cmd_forward * self.true_theta.sin() * dt;

        // Bounce off walls (makes it more interesting than wrapping)
        if self.true_pos.x < 0.05 || self.true_pos.x > 0.95 {
            self.true_theta = std::f32::consts::PI - self.true_theta;
            self.true_pos.x = self.true_pos.x.clamp(0.05, 0.95);
        }
        if self.true_pos.y < 0.05 || self.true_pos.y > 0.95 {
            self.true_theta = -self.true_theta;
            self.true_pos.y = self.true_pos.y.clamp(0.05, 0.95);
        }
    }

    /// PREDICT: Move particles according to motion command + noise
    fn predict(&mut self, dt: f32) {
        // Pre-generate noise values to avoid borrow conflicts
        let n = self.particles.len();
        let mut turn_noise: Vec<f32> = Vec::with_capacity(n);
        let mut forward_noise: Vec<f32> = Vec::with_capacity(n);

        for _ in 0..n {
            turn_noise.push(self.gaussian(0.0, self.motion_noise * 2.0));
            forward_noise.push(self.gaussian(0.0, self.motion_noise));
        }

        for (i, particle) in self.particles.iter_mut().enumerate() {
            particle.prev_pos = particle.pos;

            // Apply same motion command as robot, but with Gaussian noise
            // This models odometry uncertainty
            let noisy_turn = self.cmd_turn + turn_noise[i];
            let noisy_forward = self.cmd_forward + forward_noise[i];

            particle.theta += noisy_turn * dt;
            particle.theta = particle.theta.rem_euclid(std::f32::consts::TAU);

            particle.pos.x += noisy_forward * particle.theta.cos() * dt;
            particle.pos.y += noisy_forward * particle.theta.sin() * dt;

            // Keep particles in bounds (soft boundary)
            particle.pos.x = particle.pos.x.clamp(0.02, 0.98);
            particle.pos.y = particle.pos.y.clamp(0.02, 0.98);
        }
    }

    /// Take sensor measurements from true robot to landmarks
    fn sense(&mut self) {
        self.measurements.clear();

        // Pre-compute landmarks and noise to avoid borrow conflicts
        let landmarks: Vec<(usize, Vec2)> = self.landmarks.iter().copied().enumerate().collect();
        let mut noises: Vec<f32> = Vec::with_capacity(landmarks.len());
        for _ in 0..landmarks.len() {
            noises.push(self.gaussian(0.0, self.sensor_noise));
        }

        for (i, (idx, lm)) in landmarks.iter().enumerate() {
            let true_range = self.true_pos.distance(*lm);
            // Add measurement noise
            let noisy_range = true_range + noises[i];

            self.measurements.push(Measurement {
                landmark_idx: *idx,
                range: true_range,
                noisy_range: noisy_range.max(0.0), // range can't be negative
            });
        }
    }

    /// UPDATE: Compute particle weights based on sensor measurements
    fn update(&mut self) {
        for particle in &mut self.particles {
            let mut log_prob = 0.0;

            // For each measurement, compute likelihood
            for meas in &self.measurements {
                let lm = self.landmarks[meas.landmark_idx];
                let pred_range = particle.pos.distance(lm);
                let diff = pred_range - meas.noisy_range;

                // Gaussian likelihood in log space (more numerically stable)
                let sigma_sq = self.sensor_noise * self.sensor_noise;
                log_prob += -diff * diff / (2.0 * sigma_sq);
            }

            // Convert back from log space
            particle.weight = log_prob.exp().max(1e-300);
        }

        // Normalize weights to sum to 1
        let sum: f32 = self.particles.iter().map(|p| p.weight).sum();
        if sum > 1e-10 {
            for particle in &mut self.particles {
                particle.weight /= sum;
            }
        } else {
            // All particles have near-zero weight - reinitialize uniformly
            let uniform = 1.0 / self.particles.len() as f32;
            for particle in &mut self.particles {
                particle.weight = uniform;
            }
        }

        self.compute_statistics();
    }

    /// Compute effective particle count and other statistics
    fn compute_statistics(&mut self) {
        // Effective particle count: N_eff = 1 / sum(w^2)
        // Measures how many particles are actually contributing
        let sum_sq: f32 = self.particles.iter().map(|p| p.weight * p.weight).sum();
        self.effective_particles = if sum_sq > 1e-10 {
            1.0 / sum_sq
        } else {
            self.particles.len() as f32
        };

        self.best_particle_weight = self.particles.iter()
            .map(|p| p.weight)
            .fold(0.0_f32, f32::max);
    }

    /// ESTIMATE: Compute weighted average position from particles
    fn estimate(&mut self) {
        self.est_pos = Vec2::ZERO;

        // Circular mean for angle (handles wraparound)
        let mut sin_sum = 0.0_f32;
        let mut cos_sum = 0.0_f32;

        for particle in &self.particles {
            self.est_pos.x += particle.pos.x * particle.weight;
            self.est_pos.y += particle.pos.y * particle.weight;
            sin_sum += particle.theta.sin() * particle.weight;
            cos_sum += particle.theta.cos() * particle.weight;
        }

        self.est_theta = sin_sum.atan2(cos_sum);

        // Track error over time
        let error = self.error();
        self.error_history.push(error);
        if self.error_history.len() > 200 {
            self.error_history.remove(0);
        }
    }

    /// RESAMPLE: Replace particles using low-variance resampling
    fn resample(&mut self) {
        if self.particles.is_empty() {
            return;
        }

        // Only resample if effective particles drops below threshold
        // This prevents particle depletion when weights are already uniform
        let threshold = self.particles.len() as f32 * 0.5;
        if self.effective_particles > threshold {
            return; // weights are diverse enough, skip resampling
        }

        let n = self.particles.len();
        let mut new_particles = Vec::with_capacity(n);

        // Low-variance resampling algorithm
        // More deterministic than multinomial resampling, preserves diversity better
        let step = 1.0 / n as f32;
        let mut r = self.rng.range(0.0, step);
        let mut c = self.particles[0].weight;
        let mut i = 0;

        let uniform_weight = 1.0 / n as f32;

        for _ in 0..n {
            while r > c && i < n - 1 {
                i += 1;
                c += self.particles[i].weight;
            }

            // Copy the selected particle
            let mut new_p = self.particles[i];
            new_p.weight = uniform_weight;
            // Add small noise to prevent particle depletion
            new_p.pos.x += self.gaussian(0.0, 0.005);
            new_p.pos.y += self.gaussian(0.0, 0.005);
            new_p.theta += self.gaussian(0.0, 0.05);
            new_particles.push(new_p);

            r += step;
        }

        self.particles = new_particles;
        self.compute_statistics();
    }

    /// Get localization error (distance between true and estimated pose)
    pub fn error(&self) -> f32 {
        self.true_pos.distance(self.est_pos)
    }

    /// Run one complete cycle of the particle filter
    fn full_cycle(&mut self, dt: f32) {
        // Move robot
        self.move_robot(dt);

        // Take measurements
        self.sense();

        // Particle filter steps
        self.predict(dt);
        self.update();
        self.estimate();
        self.resample();
    }

    /// Advance to next phase (for step mode)
    pub fn next_phase(&mut self, dt: f32) {
        match self.phase {
            PFPhase::Predict => {
                self.move_robot(dt);
                self.sense();
                self.predict(dt);
            }
            PFPhase::Update => {
                self.update();
            }
            PFPhase::Resample => {
                self.resample();
            }
            PFPhase::Estimate => {
                self.estimate();
            }
        }
        self.phase = self.phase.next();
    }
}

impl Demo for ParticleFilterDemo {
    fn reset(&mut self, seed: u64) {
        self.rng = Rng::new(seed);
        self.time = 0.0;
        self.phase = PFPhase::Predict;
        self.phase_timer = 0.0;
        self.error_history.clear();

        // Reset true pose to a random position
        self.true_pos = Vec2::new(
            self.rng.range(0.2, 0.8),
            self.rng.range(0.2, 0.8),
        );
        self.true_prev_pos = self.true_pos;
        self.true_theta = self.rng.range(0.0, std::f32::consts::TAU);

        // Randomize motion commands slightly
        self.cmd_forward = 0.15;
        self.cmd_turn = self.rng.range(-0.8, 0.8);

        // Initialize landmarks (known map - robot knows where these are)
        self.landmarks = vec![
            Vec2::new(0.1, 0.1),   // corners
            Vec2::new(0.9, 0.1),
            Vec2::new(0.9, 0.9),
            Vec2::new(0.1, 0.9),
            Vec2::new(0.5, 0.1),   // midpoints of edges
            Vec2::new(0.5, 0.9),
            Vec2::new(0.1, 0.5),
            Vec2::new(0.9, 0.5),
        ];

        // Initialize particles (uniformly distributed - robot doesn't know where it is)
        self.init_particles();

        // Take initial measurements
        self.sense();

        // Initial estimate
        self.estimate();
    }

    fn step(&mut self, dt: f32) {
        self.time += dt;

        // Occasionally change direction to make it more interesting
        if self.rng.range(0.0, 1.0) < 0.005 {
            self.cmd_turn = self.rng.range(-1.0, 1.0);
        }

        if self.step_mode {
            // In step mode, phases are advanced manually via next_phase()
            // Just increment timer for display purposes
            self.phase_timer += dt;
        } else {
            // Continuous mode: run full cycle each frame
            self.full_cycle(dt);
        }
    }

    fn set_param(&mut self, name: &str, value: f32) -> bool {
        match name {
            "num_particles" => {
                self.num_particles = (value as usize).clamp(10, 500);
                self.init_particles();
                true
            }
            "motion_noise" => {
                self.motion_noise = value.clamp(0.0, 0.1);
                true
            }
            "sensor_noise" => {
                self.sensor_noise = value.clamp(0.01, 0.15);
                true
            }
            "step_mode" => {
                self.step_mode = value > 0.5;
                true
            }
            _ => false,
        }
    }

    fn params() -> &'static [ParamMeta] {
        &[
            ParamMeta {
                name: "num_particles",
                label: "Particle Count",
                min: 10.0,
                max: 500.0,
                step: 10.0,
                default: 500.0,   // BEST: maximum particles
            },
            ParamMeta {
                name: "motion_noise",
                label: "Motion Noise",
                min: 0.0,
                max: 0.1,
                step: 0.005,
                default: 0.0,     // BEST: no noise
            },
            ParamMeta {
                name: "sensor_noise",
                label: "Sensor Noise",
                min: 0.01,
                max: 0.15,
                step: 0.005,
                default: 0.01,    // BEST: minimum noise
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reset_initializes_particles() {
        let mut demo = ParticleFilterDemo::default();
        demo.reset(42);
        assert_eq!(demo.particles.len(), 500);  // BEST: default max particles
    }

    #[test]
    fn test_weights_normalized() {
        let mut demo = ParticleFilterDemo::default();
        demo.reset(42);

        // Run a few steps
        for _ in 0..10 {
            demo.step(0.016);
        }

        let sum: f32 = demo.particles.iter().map(|p| p.weight).sum();
        assert!(
            (sum - 1.0).abs() < 0.01,
            "Weights should sum to 1: {}",
            sum
        );
    }

    #[test]
    fn test_particles_converge() {
        let mut demo = ParticleFilterDemo::default();
        demo.sensor_noise = 0.02; // Lower noise for faster convergence
        demo.num_particles = 200;
        demo.reset(42);

        // Run many steps
        for _ in 0..100 {
            demo.step(0.016);
        }

        // Error should be reasonably small
        let error = demo.error();
        assert!(
            error < 0.2,
            "Localization error should be small: {}",
            error
        );
    }

    #[test]
    fn test_deterministic() {
        let mut demo1 = ParticleFilterDemo::default();
        let mut demo2 = ParticleFilterDemo::default();

        demo1.reset(123);
        demo2.reset(123);

        for _ in 0..10 {
            demo1.step(0.016);
            demo2.step(0.016);
        }

        assert!(
            (demo1.true_pos.x - demo2.true_pos.x).abs() < 1e-6,
            "Should be deterministic"
        );
    }
}
