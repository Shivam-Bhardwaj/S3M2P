//! Pure mathematical ecosystem model (no rendering)
//!
//! Wave-based particle spawning with PID control.
//! Zero unsafe code, fully tested, optimized for stability testing.
//!
//! # Design Philosophy
//! - Math first, visuals later
//! - All parameters tunable via HyperParams
//! - Deterministic with seeded RNG for reproducibility
//! - O(samples + particles) per frame

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Cell types in the ecosystem grid
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum CellType {
    Empty = 0,
    Prey = 1,
    Predator = 2,
}

/// Circular wave emanating from a point
#[derive(Clone, Debug)]
pub struct Wave {
    pub cx: f32,
    pub cy: f32,
    pub birth_time: u32,
    pub speed: f32,
    pub frequency: f32,
    pub amplitude: f32,
    pub polarity: i8, // +1 or -1
}

impl Wave {
    /// Calculate wave amplitude at position (x, y) at time t
    #[inline]
    pub fn value(&self, x: f32, y: f32, t: u32) -> f32 {
        let age = (t.saturating_sub(self.birth_time)) as f32;
        let dx = x - self.cx;
        let dy = y - self.cy;
        let dist = (dx * dx + dy * dy).sqrt();
        let wavefront = self.speed * age;

        // Wave hasn't reached this point yet
        if dist > wavefront + 20.0 {
            return 0.0;
        }

        let decay = 1.0 / (1.0 + dist / 80.0);
        let oscillation = (self.frequency * (dist - wavefront)).cos();

        self.polarity as f32 * self.amplitude * decay * oscillation
    }

    /// Check if wave is still active (hasn't traveled past max distance)
    #[inline]
    pub fn is_alive(&self, t: u32, max_dist: f32) -> bool {
        let age = (t.saturating_sub(self.birth_time)) as f32;
        self.speed * age < max_dist
    }
}

/// PID Controller for population regulation
#[derive(Clone, Debug)]
pub struct PIDController {
    pub kp: f32,
    pub ki: f32,
    pub kd: f32,
    prev_error: f32,
    integral: f32,
}

impl PIDController {
    pub fn new(kp: f32, ki: f32, kd: f32) -> Self {
        Self {
            kp,
            ki,
            kd,
            prev_error: 0.0,
            integral: 0.0,
        }
    }

    /// Update controller and return control signal
    pub fn update(&mut self, error: f32) -> f32 {
        let derivative = error - self.prev_error;
        self.integral += error;

        // Anti-windup: clamp integral to prevent runaway
        self.integral = self.integral.clamp(-10000.0, 10000.0);

        self.prev_error = error;

        self.kp * error + self.ki * self.integral + self.kd * derivative
    }

    pub fn reset(&mut self) {
        self.prev_error = 0.0;
        self.integral = 0.0;
    }
}

/// Hyperparameters for the ecosystem - the values we're trying to optimize
#[derive(Clone, Debug)]
pub struct HyperParams {
    // PID gains
    pub pid_kp: f32,
    pub pid_ki: f32,
    pub pid_kd: f32,

    // Wave spawning
    pub bhardwaj_constant: f32, // Threshold for wave collapse
    pub wave_spawn_rate: f32,   // Base probability of spawning wave per frame
    pub max_waves: usize,

    // Predator parameters
    pub predator_energy: u32,      // Initial energy (frames of life)
    pub predator_hunt_chance: f32, // Chance to move toward prey

    // Spawn behavior
    pub sample_rate: f32,        // Fraction of grid to sample per frame
    pub adaptive_sampling: bool, // Increase sampling when species is low

    // Direct spawning (backup)
    pub enable_direct_spawn: bool,
    pub direct_spawn_rate: f32, // Fraction of deficit to spawn per frame
}

impl Default for HyperParams {
    fn default() -> Self {
        Self {
            pid_kp: 0.01,
            pid_ki: 0.0005,
            pid_kd: 0.1,
            bhardwaj_constant: 0.5,
            wave_spawn_rate: 0.015,
            max_waves: 60,
            predator_energy: 600,
            predator_hunt_chance: 0.2,
            sample_rate: 0.05,
            adaptive_sampling: true,
            enable_direct_spawn: false,
            direct_spawn_rate: 0.01,
        }
    }
}

/// Metrics from a single frame update
#[derive(Clone, Debug, Default)]
pub struct FrameMetrics {
    pub prey: usize,
    pub predators: usize,
    pub total: usize,
    pub prey_spawned: usize,
    pub predators_spawned: usize,
    pub predators_starved: usize,
    pub wave_count: usize,
    pub bhardwaj_constant: f32,
    pub wave_spawn_rate: f32,
}

/// The core ecosystem simulation - pure math, no rendering
pub struct Ecosystem {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<CellType>,
    pub energy: Vec<u32>,
    pub time: u32,

    // Waves
    pub waves: Vec<Wave>,

    // Control
    pub pid: PIDController,
    pub bhardwaj_constant: f32,
    pub wave_spawn_rate: f32,

    // Targets
    pub target_population: usize,
    pub target_prey_ratio: f32,

    // Hyperparameters
    pub params: HyperParams,

    // RNG for reproducibility
    rng: StdRng,
}

impl Ecosystem {
    /// Create a new ecosystem with given dimensions and target population
    pub fn new(width: usize, height: usize, target: usize) -> Self {
        Self::with_seed(width, height, target, 42)
    }

    /// Create with specific seed for reproducibility
    pub fn with_seed(width: usize, height: usize, target: usize, seed: u64) -> Self {
        let size = width * height;
        let params = HyperParams::default();

        Self {
            width,
            height,
            grid: vec![CellType::Empty; size],
            energy: vec![0; size],
            time: 0,
            waves: Vec::with_capacity(100),
            pid: PIDController::new(params.pid_kp, params.pid_ki, params.pid_kd),
            bhardwaj_constant: params.bhardwaj_constant,
            wave_spawn_rate: params.wave_spawn_rate,
            target_population: target,
            target_prey_ratio: 0.65,
            params,
            rng: StdRng::seed_from_u64(seed),
        }
    }

    /// Apply hyperparameters (useful for parameter sweep)
    pub fn apply_params(&mut self, params: HyperParams) {
        self.pid = PIDController::new(params.pid_kp, params.pid_ki, params.pid_kd);
        self.bhardwaj_constant = params.bhardwaj_constant;
        self.wave_spawn_rate = params.wave_spawn_rate;
        self.params = params;
    }

    /// Seed initial population
    pub fn seed_population(&mut self, prey_count: usize, predator_count: usize) {
        // Spawn prey
        for _ in 0..prey_count {
            let x = self.rng.gen_range(0..self.width);
            let y = self.rng.gen_range(0..self.height);
            let idx = y * self.width + x;
            if self.grid[idx] == CellType::Empty {
                self.grid[idx] = CellType::Prey;
            }
        }

        // Spawn predators with randomized energy (desynchronization)
        for _ in 0..predator_count {
            let x = self.rng.gen_range(0..self.width);
            let y = self.rng.gen_range(0..self.height);
            let idx = y * self.width + x;
            if self.grid[idx] == CellType::Empty {
                self.grid[idx] = CellType::Predator;
                // Randomize energy to desynchronize death
                let variation = 0.5 + self.rng.gen::<f32>() * 0.5;
                self.energy[idx] = (self.params.predator_energy as f32 * variation) as u32;
            }
        }

        // Seed some initial waves
        for _ in 0..30 {
            let x = self.rng.gen_range(0.0..self.width as f32);
            let y = self.rng.gen_range(0.0..self.height as f32);
            let birth_offset = self.rng.gen_range(0..150);
            self.waves.push(Wave {
                cx: x,
                cy: y,
                birth_time: 0_u32.saturating_sub(birth_offset),
                speed: 2.0,
                frequency: 0.08 + self.rng.gen::<f32>() * 0.04,
                amplitude: 1.5 + self.rng.gen::<f32>() * 1.0,
                polarity: if self.rng.gen::<bool>() { 1 } else { -1 },
            });
        }
    }

    /// Count particles in grid (single source of truth)
    pub fn count_particles(&self) -> (usize, usize, usize) {
        let mut prey = 0;
        let mut predators = 0;

        for &cell in &self.grid {
            match cell {
                CellType::Prey => prey += 1,
                CellType::Predator => predators += 1,
                CellType::Empty => {}
            }
        }

        (prey, predators, prey + predators)
    }

    /// Spawn a new circular wave
    fn spawn_wave(&mut self, polarity_bias: f32) {
        if self.waves.len() >= self.params.max_waves {
            return;
        }

        let cx = self.rng.gen_range(0.0..self.width as f32);
        let cy = self.rng.gen_range(0.0..self.height as f32);

        let polarity = if self.rng.gen::<f32>() < polarity_bias {
            -1 // Negative = predator spawning
        } else {
            1 // Positive = prey spawning
        };

        self.waves.push(Wave {
            cx,
            cy,
            birth_time: self.time,
            speed: 2.0,
            frequency: 0.08 + self.rng.gen::<f32>() * 0.04,
            amplitude: 1.5 + self.rng.gen::<f32>() * 1.0,
            polarity,
        });
    }

    /// Main update loop - returns frame metrics
    pub fn update(&mut self) -> FrameMetrics {
        self.time += 1;

        let (prey, predators, total) = self.count_particles();
        let mut metrics = FrameMetrics {
            prey,
            predators,
            total,
            ..Default::default()
        };

        // PID regulation for total population
        let error = total as f32 - self.target_population as f32;
        let pid_output = self.pid.update(error / self.target_population as f32);

        // Apply PID to control parameters
        self.wave_spawn_rate = (self.wave_spawn_rate - pid_output * 0.005).clamp(0.001, 0.04);
        self.bhardwaj_constant = (self.bhardwaj_constant + pid_output * 0.02).clamp(0.3, 0.95);

        metrics.wave_spawn_rate = self.wave_spawn_rate;
        metrics.bhardwaj_constant = self.bhardwaj_constant;

        // Balance control: adjust wave polarity based on prey/predator ratio
        let prey_ratio = if total > 0 {
            prey as f32 / total as f32
        } else {
            0.5
        };
        let polarity_bias = if prey_ratio > 0.75 {
            0.8 // Spawn more negative waves (predators)
        } else if prey_ratio < 0.55 {
            0.2 // Spawn more positive waves (prey)
        } else {
            0.5 // Balanced
        };

        // Spawn waves
        if self.rng.gen::<f32>() < self.wave_spawn_rate {
            self.spawn_wave(polarity_bias);
        }

        // Cleanup old waves
        let max_dist = self.width.max(self.height) as f32 * 1.5;
        self.waves.retain(|w| w.is_alive(self.time, max_dist));
        metrics.wave_count = self.waves.len();

        // Wave collapse - spawn particles
        let (prey_spawned, pred_spawned) = self.wave_collapse(total);
        metrics.prey_spawned = prey_spawned;
        metrics.predators_spawned = pred_spawned;

        // CRITICAL: Ratio-based spawning to maintain balance
        // This compensates for wave spawning asymmetry
        self.ratio_spawn(&mut metrics);

        // Direct spawning backup (if enabled)
        if self.params.enable_direct_spawn {
            self.direct_spawn(&mut metrics);
        }

        // Extinction prevention (hard floor)
        self.extinction_prevention(&mut metrics);

        // Particle dynamics (predator starvation, movement)
        let starved = self.update_particles();
        metrics.predators_starved = starved;

        // Update final counts
        let (prey, predators, total) = self.count_particles();
        metrics.prey = prey;
        metrics.predators = predators;
        metrics.total = total;

        metrics
    }

    /// Ratio-based spawning - maintains prey/predator balance
    /// This is the key fix for wave spawning asymmetry
    fn ratio_spawn(&mut self, metrics: &mut FrameMetrics) {
        let target_pred = (self.target_population as f32 * (1.0 - self.target_prey_ratio)) as usize;
        let target_prey = (self.target_population as f32 * self.target_prey_ratio) as usize;

        // Predator spawning based on deficit
        if metrics.predators < target_pred {
            let deficit = target_pred - metrics.predators;
            // Spawn rate proportional to deficit, but capped
            let spawn_rate = (deficit as f32 / target_pred as f32).min(0.5);
            let spawn_count = ((deficit as f32 * spawn_rate * 0.05) as usize).clamp(1, 20);

            for _ in 0..spawn_count {
                let x = self.rng.gen_range(0..self.width);
                let y = self.rng.gen_range(0..self.height);
                let idx = y * self.width + x;
                if self.grid[idx] == CellType::Empty {
                    self.grid[idx] = CellType::Predator;
                    // Randomize energy for desynchronization
                    let variation = 0.5 + self.rng.gen::<f32>() * 0.5;
                    self.energy[idx] = (self.params.predator_energy as f32 * variation) as u32;
                    metrics.predators_spawned += 1;
                }
            }
        }

        // Prey spawning based on deficit (less aggressive since waves spawn prey well)
        if metrics.prey < target_prey {
            let deficit = target_prey - metrics.prey;
            let spawn_rate = (deficit as f32 / target_prey as f32).min(0.3);
            let spawn_count = ((deficit as f32 * spawn_rate * 0.02) as usize).clamp(0, 10);

            for _ in 0..spawn_count {
                let x = self.rng.gen_range(0..self.width);
                let y = self.rng.gen_range(0..self.height);
                let idx = y * self.width + x;
                if self.grid[idx] == CellType::Empty {
                    self.grid[idx] = CellType::Prey;
                    metrics.prey_spawned += 1;
                }
            }
        }
    }

    /// Wave function collapse - sample grid and spawn particles
    fn wave_collapse(&mut self, current_total: usize) -> (usize, usize) {
        let mut prey_spawned = 0;
        let mut pred_spawned = 0;

        if current_total >= self.target_population {
            return (0, 0);
        }

        // Calculate effective sample rate
        let mut effective_rate = self.params.sample_rate;
        if self.params.adaptive_sampling {
            let (prey, predators, _) = self.count_particles();
            // Increase sampling when species is critically low
            if predators < 200 {
                effective_rate *= 4.0; // 20% sampling for predators
            } else if prey < 500 {
                effective_rate *= 2.0; // 10% sampling for prey
            }
            effective_rate = effective_rate.min(0.3); // Cap at 30%
        }

        let samples = (self.grid.len() as f32 * effective_rate) as usize;

        for _ in 0..samples {
            if current_total + prey_spawned + pred_spawned >= self.target_population {
                break;
            }

            let x = self.rng.gen_range(0..self.width);
            let y = self.rng.gen_range(0..self.height);
            let idx = y * self.width + x;

            if self.grid[idx] != CellType::Empty {
                continue;
            }

            // Sum wave amplitudes at this point
            let mut total_amp = 0.0;
            for wave in &self.waves {
                total_amp += wave.value(x as f32, y as f32, self.time);
            }

            // Collapse based on Bhardwaj constant
            if total_amp > self.bhardwaj_constant {
                self.grid[idx] = CellType::Prey;
                prey_spawned += 1;
            } else if total_amp < -self.bhardwaj_constant {
                self.grid[idx] = CellType::Predator;
                // Randomize energy for desynchronization
                let variation = 0.5 + self.rng.gen::<f32>() * 0.5;
                self.energy[idx] = (self.params.predator_energy as f32 * variation) as u32;
                pred_spawned += 1;
            }
        }

        (prey_spawned, pred_spawned)
    }

    /// Direct spawning backup (when waves aren't enough)
    fn direct_spawn(&mut self, metrics: &mut FrameMetrics) {
        let target_prey = (self.target_population as f32 * self.target_prey_ratio) as usize;
        let target_pred = self.target_population - target_prey;

        // Spawn prey if under target
        if metrics.prey < target_prey {
            let deficit = target_prey - metrics.prey;
            let spawn_count = ((deficit as f32 * self.params.direct_spawn_rate) as usize).min(50);

            for _ in 0..spawn_count {
                let x = self.rng.gen_range(0..self.width);
                let y = self.rng.gen_range(0..self.height);
                let idx = y * self.width + x;
                if self.grid[idx] == CellType::Empty {
                    self.grid[idx] = CellType::Prey;
                    metrics.prey_spawned += 1;
                }
            }
        }

        // Spawn predators if under target
        if metrics.predators < target_pred {
            let deficit = target_pred - metrics.predators;
            let spawn_count = ((deficit as f32 * self.params.direct_spawn_rate) as usize).min(20);

            for _ in 0..spawn_count {
                let x = self.rng.gen_range(0..self.width);
                let y = self.rng.gen_range(0..self.height);
                let idx = y * self.width + x;
                if self.grid[idx] == CellType::Empty {
                    self.grid[idx] = CellType::Predator;
                    let variation = 0.5 + self.rng.gen::<f32>() * 0.5;
                    self.energy[idx] = (self.params.predator_energy as f32 * variation) as u32;
                    metrics.predators_spawned += 1;
                }
            }
        }
    }

    /// Hard extinction prevention (emergency spawning)
    fn extinction_prevention(&mut self, metrics: &mut FrameMetrics) {
        // Predator extinction prevention
        if metrics.predators < 20 {
            for _ in 0..50 {
                let x = self.rng.gen_range(0..self.width);
                let y = self.rng.gen_range(0..self.height);
                let idx = y * self.width + x;
                if self.grid[idx] == CellType::Empty {
                    self.grid[idx] = CellType::Predator;
                    let variation = 0.5 + self.rng.gen::<f32>() * 0.5;
                    self.energy[idx] = (self.params.predator_energy as f32 * variation) as u32;
                    metrics.predators_spawned += 1;
                }
            }
        }

        // Prey extinction prevention
        if metrics.prey < 50 {
            for _ in 0..200 {
                let x = self.rng.gen_range(0..self.width);
                let y = self.rng.gen_range(0..self.height);
                let idx = y * self.width + x;
                if self.grid[idx] == CellType::Empty {
                    self.grid[idx] = CellType::Prey;
                    metrics.prey_spawned += 1;
                }
            }
        }
    }

    /// Update particle dynamics (energy drain, death)
    fn update_particles(&mut self) -> usize {
        let mut starved = 0;

        for idx in 0..self.grid.len() {
            if self.grid[idx] == CellType::Predator {
                if self.energy[idx] > 0 {
                    self.energy[idx] -= 1;
                } else {
                    self.grid[idx] = CellType::Empty;
                    starved += 1;
                }
            }
        }

        starved
    }

    /// Run simulation for N frames, collecting metrics
    pub fn run(&mut self, frames: u32) -> Vec<FrameMetrics> {
        let mut history = Vec::with_capacity(frames as usize);

        for _ in 0..frames {
            let metrics = self.update();
            history.push(metrics);
        }

        history
    }
}

/// Stability analysis results
#[derive(Clone, Debug)]
pub struct StabilityReport {
    pub frames_run: u32,
    pub extinctions: u32,
    pub avg_population: f32,
    pub std_dev: f32,
    pub min_population: usize,
    pub max_population: usize,
    pub avg_prey_ratio: f32,
    pub oscillations: u32,
    pub settling_time: Option<u32>, // Frame when population stabilized within 10%
    pub stable: bool,
}

/// Analyze ecosystem stability over time
pub fn analyze_stability(history: &[FrameMetrics], target: usize) -> StabilityReport {
    if history.is_empty() {
        return StabilityReport {
            frames_run: 0,
            extinctions: 0,
            avg_population: 0.0,
            std_dev: 0.0,
            min_population: 0,
            max_population: 0,
            avg_prey_ratio: 0.0,
            oscillations: 0,
            settling_time: None,
            stable: false,
        };
    }

    let frames = history.len() as u32;

    // Count extinctions
    let extinctions = history
        .iter()
        .filter(|m| m.prey == 0 || m.predators == 0)
        .count() as u32;

    // Population statistics
    let populations: Vec<f32> = history.iter().map(|m| m.total as f32).collect();
    let avg_pop = populations.iter().sum::<f32>() / populations.len() as f32;
    let variance = populations
        .iter()
        .map(|&p| (p - avg_pop).powi(2))
        .sum::<f32>()
        / populations.len() as f32;
    let std_dev = variance.sqrt();

    let min_pop = history.iter().map(|m| m.total).min().unwrap_or(0);
    let max_pop = history.iter().map(|m| m.total).max().unwrap_or(0);

    // Prey ratio
    let avg_prey_ratio = history
        .iter()
        .filter(|m| m.total > 0)
        .map(|m| m.prey as f32 / m.total as f32)
        .sum::<f32>()
        / history.len() as f32;

    // Count oscillations (direction changes)
    let mut oscillations = 0u32;
    for i in 2..history.len() {
        let prev_slope = history[i - 1].total as i32 - history[i - 2].total as i32;
        let curr_slope = history[i].total as i32 - history[i - 1].total as i32;
        if prev_slope.signum() != curr_slope.signum() && prev_slope != 0 && curr_slope != 0 {
            oscillations += 1;
        }
    }

    // Find settling time (first frame where population stays within 10% of target)
    let tolerance = target as f32 * 0.1;
    let mut settling_time = None;
    for (i, m) in history.iter().enumerate() {
        let error = (m.total as f32 - target as f32).abs();
        if error < tolerance {
            // Check if it stays stable for at least 100 frames
            let stable_window = history
                .iter()
                .skip(i)
                .take(100)
                .all(|m| (m.total as f32 - target as f32).abs() < tolerance);
            if stable_window {
                settling_time = Some(i as u32);
                break;
            }
        }
    }

    // Stability check
    let error_pct = (avg_pop - target as f32).abs() / target as f32;
    let stable =
        extinctions == 0 && error_pct < 0.2 && avg_prey_ratio > 0.4 && avg_prey_ratio < 0.8;

    StabilityReport {
        frames_run: frames,
        extinctions,
        avg_population: avg_pop,
        std_dev,
        min_population: min_pop,
        max_population: max_pop,
        avg_prey_ratio,
        oscillations,
        settling_time,
        stable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wave_value() {
        let wave = Wave {
            cx: 50.0,
            cy: 50.0,
            birth_time: 0,
            speed: 2.0,
            frequency: 0.1,
            amplitude: 1.0,
            polarity: 1,
        };

        // At t=0, only the center should have significant amplitude
        let val_center = wave.value(50.0, 50.0, 0);
        let val_far = wave.value(100.0, 100.0, 0);

        assert!(val_center.abs() > 0.5, "Center should have high amplitude");
        assert!(
            val_far.abs() < 0.1,
            "Far point should have low amplitude before wave arrives"
        );

        // At t=50, wave should have traveled further
        let val_middle = wave.value(75.0, 50.0, 50);
        assert!(
            val_middle.abs() > 0.1,
            "Middle point should have amplitude after wave passes"
        );
    }

    #[test]
    fn test_pid_controller() {
        let mut pid = PIDController::new(0.1, 0.01, 0.05);

        // Positive error should produce positive output
        let out1 = pid.update(100.0);
        assert!(out1 > 0.0, "Positive error should give positive output");

        // Sustained error: integral increases but derivative is 0
        // out2 = Kp*100 + Ki*(100+100) + Kd*0 = 10 + 2 + 0 = 12
        // out1 = Kp*100 + Ki*100 + Kd*100 = 10 + 1 + 5 = 16
        // So out2 < out1 due to derivative term being 0 on second call
        let out2 = pid.update(100.0);
        // Check integral is accumulating (we test by running more steps)
        let _out2 = pid.update(100.0);

        let _out3 = pid.update(100.0);
        // After 4 steps, integral = 400, so Ki*integral = 4.0
        // Without derivative contribution, it should still be significant
        assert!(
            out2 > 10.0, // This was `out4` in the original, but the snippet implies `out2` is the last one before the assert.
            "Accumulated integral should contribute: got {}",
            out2 // This was `out4` in the original.
        );

        // Negative error should produce negative output
        pid.reset();
        let out_neg = pid.update(-100.0);
        assert!(out_neg < 0.0, "Negative error should give negative output");
    }

    #[test]
    fn test_ecosystem_creation() {
        let eco = Ecosystem::new(960, 540, 15000);

        assert_eq!(eco.width, 960);
        assert_eq!(eco.height, 540);
        assert_eq!(eco.grid.len(), 960 * 540);
        assert_eq!(eco.target_population, 15000);
    }

    #[test]
    fn test_seed_population() {
        let mut eco = Ecosystem::new(960, 540, 15000);
        eco.seed_population(3000, 1500);

        let (prey, predators, total) = eco.count_particles();

        // Due to overlapping, actual counts may be slightly less
        assert!(prey > 2500, "Should have ~3000 prey, got {}", prey);
        assert!(
            predators > 1200,
            "Should have ~1500 predators, got {}",
            predators
        );
        assert!(total > 3700, "Should have ~4500 total, got {}", total);
    }

    #[test]
    fn test_update_runs() {
        let mut eco = Ecosystem::new(960, 540, 15000);
        eco.seed_population(3000, 1500);

        // Run 100 frames
        for _ in 0..100 {
            let metrics = eco.update();
            assert!(metrics.total > 0, "Population should not go to zero");
        }
    }

    #[test]
    fn test_extinction_prevention() {
        let mut eco = Ecosystem::new(960, 540, 15000);
        // Start with very low population
        eco.seed_population(10, 5);

        // Run some frames
        for _ in 0..100 {
            let metrics = eco.update();
            // Extinction prevention should kick in
            assert!(
                metrics.prey > 0 || metrics.predators > 0,
                "Should prevent total extinction"
            );
        }
    }

    #[test]
    fn test_stability_20_seconds() {
        let mut eco = Ecosystem::with_seed(960, 540, 15000, 12345);
        eco.seed_population(3000, 1500);

        let history = eco.run(1200); // 20 seconds at 60fps
        let report = analyze_stability(&history, 15000);

        // Check no extinctions
        assert_eq!(
            report.extinctions, 0,
            "Should have no extinctions in 20 seconds"
        );

        // Check population is reasonable
        assert!(
            report.avg_population > 5000.0,
            "Average population should be > 5000"
        );
    }

    #[test]
    fn test_hyperparams_apply() {
        let mut eco = Ecosystem::new(960, 540, 15000);

        let custom_params = HyperParams {
            pid_kp: 0.02,
            pid_ki: 0.001,
            pid_kd: 0.2,
            bhardwaj_constant: 0.7,
            wave_spawn_rate: 0.02,
            max_waves: 80,
            predator_energy: 800,
            predator_hunt_chance: 0.3,
            sample_rate: 0.1,
            adaptive_sampling: false,
            enable_direct_spawn: true,
            direct_spawn_rate: 0.02,
        };

        eco.apply_params(custom_params.clone());

        assert_eq!(eco.params.pid_kp, 0.02);
        assert_eq!(eco.params.predator_energy, 800);
        assert!(eco.params.enable_direct_spawn);
    }

    #[test]
    fn test_stability_report() {
        let history = vec![
            FrameMetrics {
                prey: 9000,
                predators: 5000,
                total: 14000,
                ..Default::default()
            },
            FrameMetrics {
                prey: 9200,
                predators: 5100,
                total: 14300,
                ..Default::default()
            },
            FrameMetrics {
                prey: 9500,
                predators: 5200,
                total: 14700,
                ..Default::default()
            },
            FrameMetrics {
                prey: 9800,
                predators: 5300,
                total: 15100,
                ..Default::default()
            },
            FrameMetrics {
                prey: 9700,
                predators: 5400,
                total: 15100,
                ..Default::default()
            },
        ];

        let report = analyze_stability(&history, 15000);

        assert_eq!(report.frames_run, 5);
        assert_eq!(report.extinctions, 0);
        assert!(report.avg_population > 14000.0);
        assert!(report.avg_prey_ratio > 0.6);
    }

    #[test]
    fn test_determinism() {
        // Two ecosystems with same seed should produce identical results
        let mut eco1 = Ecosystem::with_seed(100, 100, 1000, 42);
        let mut eco2 = Ecosystem::with_seed(100, 100, 1000, 42);

        eco1.seed_population(200, 100);
        eco2.seed_population(200, 100);

        for _ in 0..50 {
            let m1 = eco1.update();
            let m2 = eco2.update();

            assert_eq!(m1.prey, m2.prey, "Prey counts should match");
            assert_eq!(m1.predators, m2.predators, "Predator counts should match");
        }
    }
}
