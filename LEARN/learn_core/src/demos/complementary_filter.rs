//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: complementary_filter.rs | LEARN/learn_core/src/demos/complementary_filter.rs
//! PURPOSE: Complementary Filter demo for IMU sensor fusion (accelerometer + gyroscope)
//! MODIFIED: 2025-12-12
//! LAYER: LEARN → learn_core → demos
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! # Why Sensor Fusion?
//!
//! Imagine measuring the tilt angle of a robot. You have two sensors:
//!
//! **Accelerometer**: Measures gravity direction → gives you absolute angle
//!   - ✓ No drift over time (gravity doesn't change)
//!   - ✗ Very noisy/jittery (vibrations, movement)
//!
//! **Gyroscope**: Measures rotation rate → integrate to get angle change
//!   - ✓ Smooth, responds quickly to motion
//!   - ✗ Drifts over time (small errors accumulate)
//!
//! **Key insight**: These weaknesses are COMPLEMENTARY!
//!   - Accelerometer is good for LOW frequency (long-term truth)
//!   - Gyroscope is good for HIGH frequency (short-term changes)
//!
//! The complementary filter combines them:
//!   angle = α × (angle + gyro×dt) + (1-α) × accel_angle
//!
//! α (alpha) is typically 0.96-0.98:
//!   - High α: Trust gyro more (smoother, but may drift)
//!   - Low α: Trust accel more (no drift, but jittery)

use crate::{Demo, ParamMeta, Rng};

/// Simulated IMU sensor readings
#[derive(Clone, Copy, Debug, Default)]
pub struct ImuReading {
    /// Angle from accelerometer (noisy but no drift)
    pub accel_angle: f32,
    /// Angular velocity from gyroscope
    pub gyro_rate: f32,
    /// Integrated gyro angle (smooth but drifts)
    pub gyro_angle: f32,
    /// Fused angle from complementary filter
    pub fused_angle: f32,
}

/// History of readings for visualization
#[derive(Clone, Debug, Default)]
pub struct SensorHistory {
    pub accel: Vec<f32>,
    pub gyro: Vec<f32>,
    pub fused: Vec<f32>,
    pub true_angle: Vec<f32>,
}

/// Complementary Filter demo for IMU sensor fusion
#[derive(Clone)]
pub struct ComplementaryFilterDemo {
    // True angle (hidden, what we're trying to estimate)
    pub true_angle: f32,

    // Current sensor readings
    pub reading: ImuReading,

    // Filter parameter
    pub alpha: f32, // 0-1, higher = trust gyro more

    // Noise parameters
    pub accel_noise: f32,  // Standard deviation of accelerometer noise
    pub gyro_drift: f32,   // Gyroscope drift rate
    pub gyro_noise: f32,   // Gyroscope noise

    // Internal state
    gyro_bias: f32,        // Accumulated gyro drift
    integrated_gyro: f32,  // Pure gyro integration (for comparison)

    // History for plotting
    pub history: SensorHistory,
    max_history: usize,

    // Motion parameters
    pub motion_frequency: f32,  // How fast the angle changes
    pub motion_amplitude: f32,  // How much the angle changes

    // Time
    time: f32,

    // RNG
    rng: Rng,
}

impl Default for ComplementaryFilterDemo {
    fn default() -> Self {
        Self {
            true_angle: 0.0,
            reading: ImuReading::default(),
            alpha: 0.98,           // Optimal blend - best of both worlds
            accel_noise: 1.0,      // BEST: minimum noise
            gyro_drift: 0.0,       // BEST: no drift
            gyro_noise: 0.5,       // degrees per second of noise
            gyro_bias: 0.0,
            integrated_gyro: 0.0,
            history: SensorHistory::default(),
            max_history: 200,
            motion_frequency: 0.3,
            motion_amplitude: 30.0,
            time: 0.0,
            rng: Rng::new(42),
        }
    }
}

impl ComplementaryFilterDemo {
    /// Sample from Gaussian distribution
    fn gaussian(&mut self, std_dev: f32) -> f32 {
        let u1 = self.rng.range(0.0001, 1.0);
        let u2 = self.rng.range(0.0, 1.0);
        let z = (-2.0 * u1.ln()).sqrt() * (std::f32::consts::TAU * u2).cos();
        std_dev * z
    }

    /// Simulate true angle motion (smooth sinusoidal + some randomness)
    fn update_true_angle(&mut self, dt: f32) {
        self.time += dt;

        // Smooth motion with occasional direction changes
        let base = self.motion_amplitude * (self.time * self.motion_frequency).sin();
        let secondary = (self.motion_amplitude * 0.3) * (self.time * self.motion_frequency * 2.3).cos();

        self.true_angle = base + secondary;
    }

    /// Simulate accelerometer reading (noisy but no drift)
    fn read_accelerometer(&mut self) -> f32 {
        // Accelerometer gives absolute angle from gravity
        // but with high-frequency noise from vibrations
        self.true_angle + self.gaussian(self.accel_noise)
    }

    /// Simulate gyroscope reading (smooth but drifts)
    fn read_gyroscope(&mut self, dt: f32, true_rate: f32) -> f32 {
        // Gyroscope measures angular velocity
        // Small noise + drift that accumulates over time
        self.gyro_bias += self.gyro_drift * dt * self.rng.range(-0.5, 1.5);

        true_rate + self.gyro_noise * self.gaussian(1.0) + self.gyro_bias
    }

    /// Apply complementary filter
    fn complementary_filter(&mut self, accel_angle: f32, gyro_rate: f32, dt: f32) -> f32 {
        // The magic formula:
        // angle = α × (previous_angle + gyro×dt) + (1-α) × accel_angle
        //
        // This is equivalent to a high-pass filter on gyro + low-pass filter on accel

        let gyro_contribution = self.reading.fused_angle + gyro_rate * dt;
        let accel_contribution = accel_angle;

        self.alpha * gyro_contribution + (1.0 - self.alpha) * accel_contribution
    }

    /// Get error between fused estimate and true angle
    pub fn fusion_error(&self) -> f32 {
        (self.reading.fused_angle - self.true_angle).abs()
    }

    /// Get error for raw accelerometer
    pub fn accel_error(&self) -> f32 {
        (self.reading.accel_angle - self.true_angle).abs()
    }

    /// Get error for raw gyro integration
    pub fn gyro_error(&self) -> f32 {
        (self.reading.gyro_angle - self.true_angle).abs()
    }
}

impl Demo for ComplementaryFilterDemo {
    fn reset(&mut self, seed: u64) {
        self.rng = Rng::new(seed);
        self.time = 0.0;
        self.true_angle = 0.0;
        self.gyro_bias = 0.0;
        self.integrated_gyro = 0.0;

        self.reading = ImuReading {
            accel_angle: 0.0,
            gyro_rate: 0.0,
            gyro_angle: 0.0,
            fused_angle: 0.0,
        };

        self.history = SensorHistory {
            accel: Vec::with_capacity(self.max_history),
            gyro: Vec::with_capacity(self.max_history),
            fused: Vec::with_capacity(self.max_history),
            true_angle: Vec::with_capacity(self.max_history),
        };
    }

    fn step(&mut self, dt: f32) {
        // Store previous angle for rate calculation
        let prev_true = self.true_angle;

        // Update true angle
        self.update_true_angle(dt);

        // Calculate true angular rate
        let true_rate = (self.true_angle - prev_true) / dt;

        // Read sensors
        let accel_angle = self.read_accelerometer();
        let gyro_rate = self.read_gyroscope(dt, true_rate);

        // Pure gyro integration (for comparison - shows drift)
        self.integrated_gyro += gyro_rate * dt;

        // Apply complementary filter
        let fused = self.complementary_filter(accel_angle, gyro_rate, dt);

        // Update readings
        self.reading = ImuReading {
            accel_angle,
            gyro_rate,
            gyro_angle: self.integrated_gyro,
            fused_angle: fused,
        };

        // Record history
        self.history.accel.push(accel_angle);
        self.history.gyro.push(self.integrated_gyro);
        self.history.fused.push(fused);
        self.history.true_angle.push(self.true_angle);

        // Trim history
        if self.history.accel.len() > self.max_history {
            self.history.accel.remove(0);
            self.history.gyro.remove(0);
            self.history.fused.remove(0);
            self.history.true_angle.remove(0);
        }
    }

    fn set_param(&mut self, name: &str, value: f32) -> bool {
        match name {
            "alpha" => {
                self.alpha = value.clamp(0.5, 0.995);
                true
            }
            "accel_noise" => {
                self.accel_noise = value.clamp(1.0, 20.0);
                true
            }
            "gyro_drift" => {
                self.gyro_drift = value.clamp(0.0, 2.0);
                true
            }
            "motion_speed" => {
                self.motion_frequency = value.clamp(0.1, 1.0);
                true
            }
            _ => false,
        }
    }

    fn params() -> &'static [ParamMeta] {
        &[
            ParamMeta {
                name: "alpha",
                label: "Alpha (α)",
                min: 0.5,
                max: 0.995,
                step: 0.005,
                default: 0.98,  // Optimal value
            },
            ParamMeta {
                name: "accel_noise",
                label: "Accel Noise",
                min: 1.0,
                max: 20.0,
                step: 1.0,
                default: 1.0,   // BEST: minimum noise
            },
            ParamMeta {
                name: "gyro_drift",
                label: "Gyro Drift",
                min: 0.0,
                max: 2.0,
                step: 0.1,
                default: 0.0,   // BEST: no drift
            },
            ParamMeta {
                name: "motion_speed",
                label: "Motion Speed",
                min: 0.1,
                max: 1.0,
                step: 0.1,
                default: 0.3,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reset() {
        let mut demo = ComplementaryFilterDemo::default();
        demo.reset(42);
        assert_eq!(demo.true_angle, 0.0);
        assert_eq!(demo.reading.fused_angle, 0.0);
    }

    #[test]
    fn test_fusion_better_than_raw() {
        let mut demo = ComplementaryFilterDemo::default();
        demo.reset(42);

        // Run for a while
        for _ in 0..500 {
            demo.step(0.016);
        }

        // Fused should be closer to truth than raw gyro (which drifts)
        let gyro_error = demo.gyro_error();
        let fused_error = demo.fusion_error();

        assert!(
            fused_error < gyro_error,
            "Fused error {} should be less than gyro error {}",
            fused_error, gyro_error
        );
    }

    #[test]
    fn test_alpha_effect() {
        // High alpha should follow gyro more closely short-term
        let mut demo_high = ComplementaryFilterDemo::default();
        demo_high.alpha = 0.99;
        demo_high.reset(42);

        let mut demo_low = ComplementaryFilterDemo::default();
        demo_low.alpha = 0.5;
        demo_low.reset(42);

        // Run a few steps
        for _ in 0..10 {
            demo_high.step(0.016);
            demo_low.step(0.016);
        }

        // Both should produce different results with same seed
        // (demonstrating alpha has an effect)
        assert!(
            (demo_high.reading.fused_angle - demo_low.reading.fused_angle).abs() > 0.1,
            "Different alpha values should produce different results"
        );
    }

    #[test]
    fn test_deterministic() {
        let mut demo1 = ComplementaryFilterDemo::default();
        let mut demo2 = ComplementaryFilterDemo::default();

        demo1.reset(123);
        demo2.reset(123);

        for _ in 0..50 {
            demo1.step(0.016);
            demo2.step(0.016);
        }

        assert!(
            (demo1.reading.fused_angle - demo2.reading.fused_angle).abs() < 1e-6,
            "Should be deterministic"
        );
    }
}
