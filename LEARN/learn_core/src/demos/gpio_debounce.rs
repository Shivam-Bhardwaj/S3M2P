//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: gpio_debounce.rs | LEARN/learn_core/src/demos/gpio_debounce.rs
//! PURPOSE: GPIO button debouncing simulation
//! MODIFIED: 2025-12-11
//! LAYER: LEARN → learn_core → demos
//! ═══════════════════════════════════════════════════════════════════════════════

use crate::{Demo, ParamMeta, Rng};

/// GPIO debounce demo showing raw vs debounced signal
///
/// Visualizes:
/// - Raw signal timeline (with bounce noise)
/// - Debounced signal timeline (clean)
/// - LED indicator state
#[derive(Clone)]
pub struct GpioDebounceDemo {
    // Signal history (ring buffer)
    pub raw_history: Vec<bool>,
    pub debounced_history: Vec<bool>,
    history_len: usize,

    // Current states
    pub raw_state: bool,
    pub debounced_state: bool,

    // Button simulation
    button_pressed: bool,
    bounce_timer: f32,
    bounce_duration: f32,
    in_bounce: bool,

    // Debounce algorithm state
    pending_state: bool,
    stable_time: f32,
    debounce_window: f32,

    // Parameters
    bounce_severity: f32, // 0-1, affects bounce duration
    sample_rate: f32,     // Samples per second
    sample_timer: f32,

    // Time tracking
    pub time: f32,
    toggle_period: f32,

    // RNG
    rng: Rng,
}

impl Default for GpioDebounceDemo {
    fn default() -> Self {
        Self {
            raw_history: Vec::new(),
            debounced_history: Vec::new(),
            history_len: 400,
            raw_state: false,
            debounced_state: false,
            button_pressed: false,
            bounce_timer: 0.0,
            bounce_duration: 0.02,
            in_bounce: false,
            pending_state: false,
            stable_time: 0.0,
            debounce_window: 0.02,
            bounce_severity: 0.5,
            sample_rate: 1000.0,
            sample_timer: 0.0,
            time: 0.0,
            toggle_period: 2.0,
            rng: Rng::new(42),
        }
    }
}

impl GpioDebounceDemo {
    /// Simulate the raw button signal with mechanical bounce
    fn simulate_button(&mut self, dt: f32) {
        // Toggle button state periodically
        let period = self.toggle_period;
        let phase = self.time % period;

        let target_state = phase < period / 2.0;

        // Detect transition
        if target_state != self.button_pressed && !self.in_bounce {
            self.button_pressed = target_state;
            self.in_bounce = true;
            self.bounce_timer = self.bounce_duration * self.bounce_severity;
        }

        // During bounce, signal is noisy
        if self.in_bounce {
            self.bounce_timer -= dt;
            if self.bounce_timer <= 0.0 {
                self.in_bounce = false;
                self.raw_state = self.button_pressed;
            } else {
                // Random bouncing during transition
                self.raw_state = self.rng.next_bool();
            }
        } else {
            self.raw_state = self.button_pressed;
        }
    }

    /// Apply debounce algorithm
    fn debounce(&mut self, dt: f32) {
        if self.raw_state != self.pending_state {
            // State changed - reset timer
            self.pending_state = self.raw_state;
            self.stable_time = 0.0;
        } else {
            // State stable - accumulate time
            self.stable_time += dt;
            if self.stable_time >= self.debounce_window {
                self.debounced_state = self.pending_state;
            }
        }
    }

    /// Sample current states to history
    fn sample(&mut self) {
        self.raw_history.push(self.raw_state);
        self.debounced_history.push(self.debounced_state);

        // Trim history
        while self.raw_history.len() > self.history_len {
            self.raw_history.remove(0);
        }
        while self.debounced_history.len() > self.history_len {
            self.debounced_history.remove(0);
        }
    }

    /// Check if currently in a bounce transition
    pub fn is_bouncing(&self) -> bool {
        self.in_bounce
    }
}

impl Demo for GpioDebounceDemo {
    fn reset(&mut self, seed: u64) {
        self.rng = Rng::new(seed);
        self.time = 0.0;
        self.sample_timer = 0.0;

        // Clear history
        self.raw_history.clear();
        self.debounced_history.clear();

        // Reset states
        self.raw_state = false;
        self.debounced_state = false;
        self.button_pressed = false;
        self.in_bounce = false;
        self.bounce_timer = 0.0;
        self.pending_state = false;
        self.stable_time = 0.0;

        // Pre-fill history
        for _ in 0..self.history_len {
            self.raw_history.push(false);
            self.debounced_history.push(false);
        }
    }

    fn step(&mut self, dt: f32) {
        self.time += dt;

        // Simulate at sample rate
        let sample_interval = 1.0 / self.sample_rate;
        self.sample_timer += dt;

        while self.sample_timer >= sample_interval {
            self.sample_timer -= sample_interval;

            // Simulate button
            self.simulate_button(sample_interval);

            // Apply debounce
            self.debounce(sample_interval);

            // Record to history
            self.sample();
        }
    }

    fn set_param(&mut self, name: &str, value: f32) -> bool {
        match name {
            "bounce_severity" => {
                self.bounce_severity = value.clamp(0.1, 1.0);
                self.bounce_duration = 0.01 + 0.04 * self.bounce_severity;
                true
            }
            "sample_rate" => {
                self.sample_rate = value.clamp(100.0, 10000.0);
                true
            }
            "debounce_window" => {
                self.debounce_window = value.clamp(0.005, 0.1);
                true
            }
            "toggle_period" => {
                self.toggle_period = value.clamp(0.5, 5.0);
                true
            }
            _ => false,
        }
    }

    fn params() -> &'static [ParamMeta] {
        &[
            ParamMeta {
                name: "bounce_severity",
                label: "Bounce Severity",
                min: 0.1,
                max: 1.0,
                step: 0.1,
                default: 0.5,
            },
            ParamMeta {
                name: "debounce_window",
                label: "Debounce Window (s)",
                min: 0.005,
                max: 0.1,
                step: 0.005,
                default: 0.02,
            },
            ParamMeta {
                name: "toggle_period",
                label: "Toggle Period (s)",
                min: 0.5,
                max: 5.0,
                step: 0.5,
                default: 2.0,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reset_fills_history() {
        let mut demo = GpioDebounceDemo::default();
        demo.reset(42);
        assert_eq!(demo.raw_history.len(), 400);
        assert_eq!(demo.debounced_history.len(), 400);
    }

    #[test]
    fn test_debounce_filters_noise() {
        let mut demo = GpioDebounceDemo::default();
        demo.bounce_severity = 0.8;
        demo.reset(42);

        // Run for several toggle cycles
        let dt = 0.001;
        let mut raw_transitions = 0;
        let mut debounced_transitions = 0;
        let mut prev_raw = false;
        let mut prev_debounced = false;

        for _ in 0..5000 {
            demo.step(dt);

            if demo.raw_state != prev_raw {
                raw_transitions += 1;
                prev_raw = demo.raw_state;
            }
            if demo.debounced_state != prev_debounced {
                debounced_transitions += 1;
                prev_debounced = demo.debounced_state;
            }
        }

        // Debounced should have far fewer transitions than raw
        assert!(
            debounced_transitions < raw_transitions,
            "Debounced ({}) should have fewer transitions than raw ({})",
            debounced_transitions,
            raw_transitions
        );
    }

    #[test]
    fn test_deterministic() {
        let mut demo1 = GpioDebounceDemo::default();
        let mut demo2 = GpioDebounceDemo::default();

        demo1.reset(123);
        demo2.reset(123);

        for _ in 0..100 {
            demo1.step(0.016);
            demo2.step(0.016);
        }

        assert_eq!(
            demo1.raw_history.len(),
            demo2.raw_history.len(),
            "History lengths should match"
        );
    }
}
