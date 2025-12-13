//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: linear_regression.rs | LEARN/learn_core/src/demos/linear_regression.rs
//! PURPOSE: Linear regression demo with animated gradient descent
//! MODIFIED: 2025-12-11
//! LAYER: LEARN → learn_core → demos
//! ═══════════════════════════════════════════════════════════════════════════════

use crate::{Demo, ParamMeta, Rng, Vec2};

/// Linear regression demo showing gradient descent optimization
///
/// Visualizes:
/// - Randomly generated data points with noise
/// - Target line (ground truth)
/// - Learned line (model prediction)
/// - Loss trace over time
#[derive(Clone)]
pub struct LinearRegressionDemo {
    // Data points (x, y)
    points: Vec<Vec2>,
    num_points: usize,

    // Ground truth parameters
    target_w: f32,
    target_b: f32,

    // Learned parameters
    pub w: f32,
    pub b: f32,

    // Training hyperparameters
    learning_rate: f32,
    noise_level: f32,

    // Training state
    step_count: usize,
    loss_history: Vec<f32>,
    max_history: usize,

    // RNG for reproducibility
    rng: Rng,
    seed: u64,
}

impl Default for LinearRegressionDemo {
    fn default() -> Self {
        Self {
            points: Vec::with_capacity(100),
            num_points: 50,
            target_w: 2.0,
            target_b: 0.5,
            w: 0.0,
            b: 0.0,
            learning_rate: 0.1,
            noise_level: 0.2,
            step_count: 0,
            loss_history: Vec::with_capacity(200),
            max_history: 100,
            rng: Rng::new(42),
            seed: 42,
        }
    }
}

impl LinearRegressionDemo {
    /// Generate training data points
    fn generate_data(&mut self) {
        self.points.clear();
        for _ in 0..self.num_points {
            let x = self.rng.range(-1.0, 1.0);
            let noise = self.rng.range(-self.noise_level, self.noise_level);
            let y = self.target_w * x + self.target_b + noise;
            self.points.push(Vec2::new(x, y));
        }
    }

    /// Compute MSE loss
    pub fn compute_loss(&self) -> f32 {
        if self.points.is_empty() {
            return 0.0;
        }
        let n = self.points.len() as f32;
        let sum_sq_error: f32 = self
            .points
            .iter()
            .map(|p| {
                let pred = self.w * p.x + self.b;
                let error = pred - p.y;
                error * error
            })
            .sum();
        sum_sq_error / n
    }

    /// Get data points for rendering
    pub fn points(&self) -> &[Vec2] {
        &self.points
    }

    /// Get loss history for rendering
    pub fn loss_history(&self) -> &[f32] {
        &self.loss_history
    }

    /// Get current step count
    pub fn step_count(&self) -> usize {
        self.step_count
    }

    /// Get target parameters
    pub fn target(&self) -> (f32, f32) {
        (self.target_w, self.target_b)
    }
}

impl Demo for LinearRegressionDemo {
    fn reset(&mut self, seed: u64) {
        self.seed = seed;
        self.rng = Rng::new(seed);

        // Reset model parameters to random starting point
        self.w = self.rng.range(-1.0, 1.0);
        self.b = self.rng.range(-1.0, 1.0);

        // Reset training state
        self.step_count = 0;
        self.loss_history.clear();

        // Generate new data
        self.generate_data();
    }

    fn step(&mut self, _dt: f32) {
        if self.points.is_empty() {
            return;
        }

        let n = self.points.len() as f32;

        // Compute gradients using MSE loss
        let mut grad_w = 0.0;
        let mut grad_b = 0.0;

        for p in &self.points {
            let pred = self.w * p.x + self.b;
            let error = pred - p.y;
            grad_w += error * p.x;
            grad_b += error;
        }

        grad_w = 2.0 * grad_w / n;
        grad_b = 2.0 * grad_b / n;

        // Gradient descent update
        self.w -= self.learning_rate * grad_w;
        self.b -= self.learning_rate * grad_b;

        self.step_count += 1;

        // Record loss every 5 steps
        if self.step_count % 5 == 0 {
            let loss = self.compute_loss();
            self.loss_history.push(loss);

            // Trim history
            if self.loss_history.len() > self.max_history {
                self.loss_history.remove(0);
            }
        }
    }

    fn set_param(&mut self, name: &str, value: f32) -> bool {
        match name {
            "learning_rate" => {
                self.learning_rate = value.clamp(0.001, 1.0);
                true
            }
            "noise" => {
                self.noise_level = value.clamp(0.0, 1.0);
                // Regenerate data with new noise
                self.rng = Rng::new(self.seed);
                self.generate_data();
                true
            }
            "target_w" => {
                self.target_w = value;
                // Regenerate data with new target
                self.rng = Rng::new(self.seed);
                self.generate_data();
                true
            }
            "target_b" => {
                self.target_b = value;
                // Regenerate data with new target
                self.rng = Rng::new(self.seed);
                self.generate_data();
                true
            }
            _ => false,
        }
    }

    fn params() -> &'static [ParamMeta] {
        &[
            ParamMeta {
                name: "learning_rate",
                label: "Learning Rate",
                min: 0.001,
                max: 1.0,
                step: 0.01,
                default: 0.1,
            },
            ParamMeta {
                name: "noise",
                label: "Noise Level",
                min: 0.0,
                max: 1.0,
                step: 0.05,
                default: 0.2,
            },
            ParamMeta {
                name: "target_w",
                label: "Target Slope (w)",
                min: -3.0,
                max: 3.0,
                step: 0.1,
                default: 2.0,
            },
            ParamMeta {
                name: "target_b",
                label: "Target Intercept (b)",
                min: -2.0,
                max: 2.0,
                step: 0.1,
                default: 0.5,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reset_generates_data() {
        let mut demo = LinearRegressionDemo::default();
        demo.reset(42);
        assert_eq!(demo.points.len(), 50);
    }

    #[test]
    fn test_deterministic() {
        let mut demo1 = LinearRegressionDemo::default();
        let mut demo2 = LinearRegressionDemo::default();

        demo1.reset(123);
        demo2.reset(123);

        assert_eq!(demo1.points.len(), demo2.points.len());
        for (p1, p2) in demo1.points.iter().zip(demo2.points.iter()) {
            assert!((p1.x - p2.x).abs() < 1e-6);
            assert!((p1.y - p2.y).abs() < 1e-6);
        }
    }

    #[test]
    fn test_gradient_descent_reduces_loss() {
        let mut demo = LinearRegressionDemo::default();
        demo.reset(42);

        let initial_loss = demo.compute_loss();

        // Run many steps
        for _ in 0..500 {
            demo.step(0.016);
        }

        let final_loss = demo.compute_loss();
        assert!(
            final_loss < initial_loss,
            "Loss should decrease: {} -> {}",
            initial_loss,
            final_loss
        );
    }

    #[test]
    fn test_convergence() {
        let mut demo = LinearRegressionDemo::default();
        demo.target_w = 1.5;
        demo.target_b = 0.3;
        demo.noise_level = 0.0; // No noise for exact convergence
        demo.learning_rate = 0.5;
        demo.reset(42);

        // Run many steps
        for _ in 0..1000 {
            demo.step(0.016);
        }

        // Should converge to target
        assert!(
            (demo.w - demo.target_w).abs() < 0.1,
            "w should converge: {} != {}",
            demo.w,
            demo.target_w
        );
        assert!(
            (demo.b - demo.target_b).abs() < 0.1,
            "b should converge: {} != {}",
            demo.b,
            demo.target_b
        );
    }

    #[test]
    fn test_set_param() {
        let mut demo = LinearRegressionDemo::default();
        demo.reset(42);

        assert!(demo.set_param("learning_rate", 0.5));
        assert!((demo.learning_rate - 0.5).abs() < 1e-6);

        assert!(!demo.set_param("nonexistent", 1.0));
    }
}
