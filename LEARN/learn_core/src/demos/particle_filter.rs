//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: particle_filter.rs | LEARN/learn_core/src/demos/particle_filter.rs
//! PURPOSE: 2D robot localization using particle filter
//! MODIFIED: 2025-12-11
//! LAYER: LEARN → learn_core → demos
//! ═══════════════════════════════════════════════════════════════════════════════

use crate::{Demo, ParamMeta, Rng, Vec2};

/// A single particle representing a hypothesis about robot pose
#[derive(Clone, Copy, Debug)]
pub struct Particle {
    pub pos: Vec2,
    pub theta: f32,
    pub weight: f32,
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            pos: Vec2::ZERO,
            theta: 0.0,
            weight: 1.0,
        }
    }
}

/// Particle filter demo for 2D robot localization
///
/// Visualizes:
/// - True robot pose (green)
/// - Particle cloud (colored by weight)
/// - Estimated pose (cyan)
/// - Landmarks (blue squares)
#[derive(Clone)]
pub struct ParticleFilterDemo {
    // True robot state
    pub true_pos: Vec2,
    pub true_theta: f32,

    // Particles
    pub particles: Vec<Particle>,
    num_particles: usize,

    // Estimated state (weighted mean of particles)
    pub est_pos: Vec2,
    pub est_theta: f32,

    // Fixed landmarks for sensing
    pub landmarks: Vec<Vec2>,

    // Noise parameters
    motion_noise: f32,
    sensor_noise: f32,

    // Time for robot motion
    time: f32,

    // RNG
    rng: Rng,
}

impl Default for ParticleFilterDemo {
    fn default() -> Self {
        Self {
            true_pos: Vec2::new(0.5, 0.5),
            true_theta: 0.0,
            particles: Vec::new(),
            num_particles: 100,
            est_pos: Vec2::new(0.5, 0.5),
            est_theta: 0.0,
            landmarks: Vec::new(),
            motion_noise: 0.02,
            sensor_noise: 0.05,
            time: 0.0,
            rng: Rng::new(42),
        }
    }
}

impl ParticleFilterDemo {
    /// Initialize particles uniformly
    fn init_particles(&mut self) {
        self.particles.clear();
        let uniform_weight = 1.0 / self.num_particles as f32;

        for _ in 0..self.num_particles {
            self.particles.push(Particle {
                pos: Vec2::new(self.rng.range(0.0, 1.0), self.rng.range(0.0, 1.0)),
                theta: self.rng.range(0.0, std::f32::consts::TAU),
                weight: uniform_weight,
            });
        }
    }

    /// Move the true robot along a path
    fn move_robot(&mut self, dt: f32) {
        self.time += dt;

        // Circular path
        let radius = 0.25;
        let speed = 0.3;
        self.true_theta = self.time * speed;
        self.true_pos = Vec2::new(
            0.5 + radius * self.true_theta.cos(),
            0.5 + radius * self.true_theta.sin(),
        );
    }

    /// Prediction step: move particles with noise
    fn predict(&mut self, dt: f32) {
        let speed = 0.3;
        let dtheta = dt * speed;

        for particle in &mut self.particles {
            // Add noise to motion
            let noisy_dtheta = dtheta + self.rng.range(-self.motion_noise, self.motion_noise);
            particle.theta += noisy_dtheta;

            // Move in direction of heading (circular motion approximation)
            let dx = -0.25 * particle.theta.sin() * dtheta
                + self.rng.range(-self.motion_noise, self.motion_noise);
            let dy = 0.25 * particle.theta.cos() * dtheta
                + self.rng.range(-self.motion_noise, self.motion_noise);

            particle.pos.x += dx;
            particle.pos.y += dy;

            // Wrap around world boundaries
            particle.pos.x = particle.pos.x.rem_euclid(1.0);
            particle.pos.y = particle.pos.y.rem_euclid(1.0);
        }
    }

    /// Update step: compute weights from sensor measurements
    fn update(&mut self) {
        // Simulate range measurements from true pose to landmarks
        let true_ranges: Vec<f32> = self
            .landmarks
            .iter()
            .map(|lm| self.true_pos.distance(*lm))
            .collect();

        // Update particle weights based on likelihood
        for particle in &mut self.particles {
            let mut prob = 1.0;

            for (lm, &true_range) in self.landmarks.iter().zip(&true_ranges) {
                let pred_range = particle.pos.distance(*lm);
                let diff = (pred_range - true_range).abs();

                // Gaussian likelihood
                let sigma_sq = self.sensor_noise * self.sensor_noise;
                prob *= (-diff * diff / (2.0 * sigma_sq)).exp();
            }

            particle.weight = prob.max(1e-10);
        }

        // Normalize weights
        let sum: f32 = self.particles.iter().map(|p| p.weight).sum();
        if sum > 1e-10 {
            for particle in &mut self.particles {
                particle.weight /= sum;
            }
        }
    }

    /// Compute estimated pose from particle weights
    fn estimate(&mut self) {
        self.est_pos = Vec2::ZERO;
        self.est_theta = 0.0;

        // Weighted mean
        for particle in &self.particles {
            self.est_pos.x += particle.pos.x * particle.weight;
            self.est_pos.y += particle.pos.y * particle.weight;
            self.est_theta += particle.theta * particle.weight;
        }
    }

    /// Resample particles using low-variance resampling
    fn resample(&mut self) {
        if self.particles.is_empty() {
            return;
        }

        let n = self.particles.len();
        let mut new_particles = Vec::with_capacity(n);

        // Low-variance resampling
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

            new_particles.push(Particle {
                pos: self.particles[i].pos,
                theta: self.particles[i].theta,
                weight: uniform_weight,
            });
            r += step;
        }

        self.particles = new_particles;
    }

    /// Get localization error (distance between true and estimated pose)
    pub fn error(&self) -> f32 {
        self.true_pos.distance(self.est_pos)
    }
}

impl Demo for ParticleFilterDemo {
    fn reset(&mut self, seed: u64) {
        self.rng = Rng::new(seed);
        self.time = 0.0;

        // Reset true pose
        self.true_pos = Vec2::new(0.5 + 0.25, 0.5);
        self.true_theta = 0.0;

        // Initialize landmarks (fixed positions)
        self.landmarks = vec![
            Vec2::new(0.15, 0.15),
            Vec2::new(0.85, 0.15),
            Vec2::new(0.85, 0.85),
            Vec2::new(0.15, 0.85),
            Vec2::new(0.5, 0.5),
        ];

        // Initialize particles
        self.init_particles();

        // Initial estimate
        self.estimate();
    }

    fn step(&mut self, dt: f32) {
        // 1. Move the true robot
        self.move_robot(dt);

        // 2. Prediction: move particles with motion model + noise
        self.predict(dt);

        // 3. Update: compute weights from sensor measurements
        self.update();

        // 4. Estimate pose from particles
        self.estimate();

        // 5. Resample (every frame for simplicity)
        self.resample();
    }

    fn set_param(&mut self, name: &str, value: f32) -> bool {
        match name {
            "num_particles" => {
                self.num_particles = (value as usize).clamp(10, 500);
                self.init_particles();
                true
            }
            "motion_noise" => {
                self.motion_noise = value.clamp(0.0, 0.2);
                true
            }
            "sensor_noise" => {
                self.sensor_noise = value.clamp(0.01, 0.3);
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
                default: 100.0,
            },
            ParamMeta {
                name: "motion_noise",
                label: "Motion Noise",
                min: 0.0,
                max: 0.2,
                step: 0.01,
                default: 0.02,
            },
            ParamMeta {
                name: "sensor_noise",
                label: "Sensor Noise",
                min: 0.01,
                max: 0.3,
                step: 0.01,
                default: 0.05,
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
        assert_eq!(demo.particles.len(), 100);
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
