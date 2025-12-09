//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: ekf.rs
//! PATH: DNA/src/physics/solvers/filters/ekf.rs
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! PURPOSE: Extended Kalman Filter for 2D state estimation
//!
//! LAYER: DNA → PHYSICS → SOLVERS → FILTERS
//!
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │ ALGORITHM: Extended Kalman Filter                                           │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │ State vector: [x, y] position (can extend to [x, y, vx, vy])                │
//! │                                                                             │
//! │ PREDICT:                                                                    │
//! │   x̂ₖ|ₖ₋₁ = f(x̂ₖ₋₁, uₖ)       State prediction                              │
//! │   Pₖ|ₖ₋₁ = FₖPₖ₋₁Fₖᵀ + Qₖ    Covariance prediction                         │
//! │                                                                             │
//! │ UPDATE:                                                                     │
//! │   yₖ = zₖ - h(x̂ₖ|ₖ₋₁)        Innovation (measurement residual)             │
//! │   Sₖ = HₖPₖ|ₖ₋₁Hₖᵀ + Rₖ      Innovation covariance                         │
//! │   Kₖ = Pₖ|ₖ₋₁HₖᵀSₖ⁻¹         Kalman gain                                    │
//! │   x̂ₖ = x̂ₖ|ₖ₋₁ + Kₖyₖ         State update                                   │
//! │   Pₖ = (I - KₖHₖ)Pₖ|ₖ₋₁      Covariance update                              │
//! └─────────────────────────────────────────────────────────────────────────────┘
//!
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │ DATA DEFINED                                                                │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │ EKF                 Extended Kalman Filter state estimator                  │
//! └─────────────────────────────────────────────────────────────────────────────┘
//!
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │ DATA FLOW                                                                   │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │ CONSUMES:  Vec2 (measurements), Vec2 (velocity), f32 (dt)                   │
//! │ PRODUCES:  Vec2 (filtered state), Mat2 (covariance), f32 (uncertainty)      │
//! └─────────────────────────────────────────────────────────────────────────────┘
//!
//! DEPENDS ON:
//!   • math/mat.rs → Mat2 for covariance matrices
//!   • glam::Vec2 → State representation
//!
//! USED BY:
//!   • Sensor fusion applications
//!   • Position tracking with noisy GPS/IMU data
//!   • Smoothing trajectory data
//!
//! ═══════════════════════════════════════════════════════════════════════════════

// ─────────────────────────────────────────────────────────────────────────────────
// CODE BELOW - Optimized for ML development
// ─────────────────────────────────────────────────────────────────────────────────

use crate::math::mat::Mat2;
use glam::Vec2;

/// Extended Kalman Filter for 2D position estimation
///
/// State vector: [x, y] position
/// Can be extended for [x, y, vx, vy] or [x, y, theta] models
///
/// # Example
/// ```
/// use dna::physics::solvers::filters::EKF;
/// use glam::Vec2;
///
/// let mut ekf = EKF::new(Vec2::new(0.0, 0.0));
///
/// // Predict step with velocity
/// ekf.predict(Vec2::new(1.0, 0.0), 0.1);
///
/// // Update with noisy measurement
/// ekf.update(Vec2::new(0.11, 0.02));
///
/// // Get filtered state
/// let state = ekf.state();
/// ```
#[derive(Clone, Debug)]
pub struct EKF {
    /// Estimated state [x, y]
    state: Vec2,
    /// Covariance matrix P (uncertainty)
    covariance: Mat2,
    /// Process noise Q (model uncertainty)
    process_noise: Mat2,
    /// Measurement noise R (sensor uncertainty)
    measurement_noise: Mat2,
}

impl EKF {
    /// Create new EKF with initial position
    pub fn new(initial_pos: Vec2) -> Self {
        Self {
            state: initial_pos,
            covariance: Mat2::IDENTITY,
            process_noise: Mat2::diagonal(0.1, 0.1),
            measurement_noise: Mat2::diagonal(1.0, 1.0),
        }
    }

    /// Create EKF with custom noise parameters
    pub fn with_noise(initial_pos: Vec2, process_noise: f32, measurement_noise: f32) -> Self {
        Self {
            state: initial_pos,
            covariance: Mat2::IDENTITY,
            process_noise: Mat2::diagonal(process_noise, process_noise),
            measurement_noise: Mat2::diagonal(measurement_noise, measurement_noise),
        }
    }

    /// Get current estimated state
    #[inline]
    pub fn state(&self) -> Vec2 {
        self.state
    }

    /// Get current covariance (uncertainty)
    #[inline]
    pub fn covariance(&self) -> Mat2 {
        self.covariance
    }

    /// Get uncertainty magnitude (trace of covariance)
    #[inline]
    pub fn uncertainty(&self) -> f32 {
        self.covariance.trace()
    }

    /// Set process noise (higher = less trust in motion model)
    pub fn set_process_noise(&mut self, noise: f32) {
        self.process_noise = Mat2::diagonal(noise, noise);
    }

    /// Set measurement noise (higher = less trust in sensors)
    pub fn set_measurement_noise(&mut self, noise: f32) {
        self.measurement_noise = Mat2::diagonal(noise, noise);
    }

    /// Predict step: propagate state forward using velocity
    ///
    /// Linear motion model: x_new = x + v * dt
    /// Jacobian F = Identity for linear motion
    pub fn predict(&mut self, velocity: Vec2, dt: f32) {
        // State prediction: x = x + v * dt
        self.state += velocity * dt;

        // Covariance prediction: P = F * P * F' + Q
        // For linear model, F = I, so P = P + Q
        // But we scale Q by dt^2 for proper integration
        let q_scaled = self.process_noise.scale(dt * dt);
        self.covariance = self.covariance.add(q_scaled);
    }

    /// Update step: correct state using measurement
    ///
    /// Direct position measurement model: z = H * x, H = Identity
    pub fn update(&mut self, measurement: Vec2) {
        // Innovation: y = z - H * x (H = I)
        let innovation = measurement - self.state;

        // Innovation covariance: S = H * P * H' + R (H = I)
        let s = self.covariance.add(self.measurement_noise);

        // Kalman gain: K = P * H' * S^-1 (H = I)
        if let Some(s_inv) = s.inverse() {
            let k = self.covariance.mul(s_inv);

            // State update: x = x + K * y
            self.state += k.mul_vec(innovation);

            // Covariance update: P = (I - K * H) * P (H = I)
            let i_minus_k = Mat2::IDENTITY.sub(k);
            self.covariance = i_minus_k.mul(self.covariance);
        }
        // If S is singular, skip update (shouldn't happen with proper noise)
    }

    /// Combined predict and update in one step
    pub fn filter(&mut self, velocity: Vec2, measurement: Vec2, dt: f32) {
        self.predict(velocity, dt);
        self.update(measurement);
    }

    /// Reset filter to new position with high uncertainty
    pub fn reset(&mut self, position: Vec2) {
        self.state = position;
        self.covariance = Mat2::diagonal(10.0, 10.0);
    }
}

impl Default for EKF {
    fn default() -> Self {
        Self::new(Vec2::ZERO)
    }
}

/// Batch filter: smooth a sequence of noisy positions
/// Returns filtered positions with the same length as input
pub fn smooth_trajectory(
    positions: &[Vec2],
    velocities: Option<&[Vec2]>,
    dt: f32,
    process_noise: f32,
    measurement_noise: f32,
) -> Vec<Vec2> {
    if positions.is_empty() {
        return Vec::new();
    }

    let mut ekf = EKF::with_noise(positions[0], process_noise, measurement_noise);
    let mut result = Vec::with_capacity(positions.len());
    result.push(ekf.state());

    for i in 1..positions.len() {
        let vel = if let Some(vels) = velocities {
            vels.get(i - 1).copied().unwrap_or(Vec2::ZERO)
        } else {
            // Estimate velocity from position difference
            (positions[i] - positions[i - 1]) / dt
        };

        ekf.predict(vel, dt);
        ekf.update(positions[i]);
        result.push(ekf.state());
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_filter() {
        let mut ekf = EKF::new(Vec2::ZERO);

        // Move with constant velocity
        for _ in 0..10 {
            ekf.predict(Vec2::new(1.0, 0.0), 0.1);
            ekf.update(ekf.state() + Vec2::new(0.02, 0.01)); // Add noise
        }

        // Should be approximately at x=1.0 after 10 steps of v=1.0, dt=0.1
        assert!((ekf.state().x - 1.0).abs() < 0.2);
    }

    #[test]
    fn test_noise_rejection() {
        let mut ekf = EKF::with_noise(Vec2::ZERO, 0.01, 10.0);

        // High measurement noise should make filter sluggish
        ekf.predict(Vec2::ZERO, 0.1);
        ekf.update(Vec2::new(100.0, 100.0)); // Outlier

        // Should not jump to outlier
        assert!(ekf.state().length() < 50.0);
    }

    #[test]
    fn test_smooth_trajectory() {
        let noisy: Vec<Vec2> = (0..20)
            .map(|i| Vec2::new(i as f32 * 0.1, 0.0) + Vec2::new(0.05, 0.05))
            .collect();

        let smooth = smooth_trajectory(&noisy, None, 0.1, 0.1, 1.0);

        assert_eq!(smooth.len(), noisy.len());
        // Smoothed trajectory should have less variation
    }

    #[test]
    fn test_uncertainty_decreases() {
        let mut ekf = EKF::new(Vec2::ZERO);
        let initial_uncertainty = ekf.uncertainty();

        // After several measurements, uncertainty should decrease
        for _ in 0..10 {
            ekf.update(Vec2::new(0.0, 0.0));
        }

        assert!(ekf.uncertainty() < initial_uncertainty);
    }
}
