// Chladni - Wave Pattern Visualization
// Rust/WASM port of realistic Chladni plate simulation
#![allow(unexpected_cfgs)]

use glam::Vec2;
use wasm_bindgen::prelude::*;

pub mod renderer;
pub mod wave;

pub use renderer::WaveRenderer;
pub use wave::{ChladniMode, WaveSimulation};

/// Chladni plate modes (m, n) - defines the vibration pattern
#[derive(Clone, Copy, Debug)]
pub struct PlateMode {
    pub m: u32, // Horizontal mode number
    pub n: u32, // Vertical mode number
}

impl PlateMode {
    pub fn new(m: u32, n: u32) -> Self {
        Self { m, n }
    }

    /// Calculate frequency for a square plate
    /// f_mn = C * (m^2 + n^2) where C depends on plate properties
    pub fn frequency(&self, plate_constant: f32) -> f32 {
        plate_constant * ((self.m * self.m + self.n * self.n) as f32)
    }
}

/// Configuration for the wave simulation
#[derive(Clone, Debug)]
pub struct SimConfig {
    pub grid_size: u32,
    pub plate_size: f32, // Physical size in meters
    pub damping: f32,    // Wave damping factor
    pub wave_speed: f32, // Wave propagation speed
    pub time_scale: f32, // Simulation speed multiplier
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            grid_size: 256,
            plate_size: 0.3, // 30cm plate
            damping: 0.002,
            wave_speed: 100.0,
            time_scale: 1.0,
        }
    }
}

/// Particle for sand visualization
#[derive(Clone, Copy, Debug)]
pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub active: bool,
}

impl Particle {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
            vel: Vec2::ZERO,
            active: true,
        }
    }
}

/// Main simulation state
pub struct ChladniSimulation {
    pub config: SimConfig,
    pub wave: WaveSimulation,
    pub particles: Vec<Particle>,
    pub current_mode: PlateMode,
    pub time: f32,
}

impl ChladniSimulation {
    pub fn new(config: SimConfig) -> Self {
        let wave = WaveSimulation::new(config.grid_size as usize);
        let particles = Self::spawn_particles(config.grid_size, 5000);

        Self {
            config,
            wave,
            particles,
            current_mode: PlateMode::new(3, 2), // Default (3,2) mode
            time: 0.0,
        }
    }

    fn spawn_particles(grid_size: u32, count: usize) -> Vec<Particle> {
        use js_sys::Math;
        let mut particles = Vec::with_capacity(count);
        let size = grid_size as f32;

        for _ in 0..count {
            let x = Math::random() as f32 * size;
            let y = Math::random() as f32 * size;
            particles.push(Particle::new(x, y));
        }

        particles
    }

    /// Update simulation by one timestep
    pub fn step(&mut self, dt: f32) {
        let dt_scaled = dt * self.config.time_scale;
        self.time += dt_scaled;

        // Update wave field
        self.wave
            .update(dt_scaled, self.current_mode, self.config.wave_speed);

        // Update particles based on wave gradient
        self.update_particles(dt_scaled);
    }

    fn update_particles(&mut self, dt: f32) {
        let grid_size = self.config.grid_size as f32;
        let damping = 0.98;

        for particle in &mut self.particles {
            if !particle.active {
                continue;
            }

            // Get wave gradient at particle position
            let gradient = self.wave.gradient_at(particle.pos.x, particle.pos.y);

            // Particles move toward nodal lines (low amplitude)
            // Force is proportional to negative gradient of amplitude squared
            let force = -gradient * 50.0;

            particle.vel += force * dt;
            particle.vel *= damping;
            particle.pos += particle.vel * dt;

            // Boundary reflection
            if particle.pos.x < 0.0 {
                particle.pos.x = 0.0;
                particle.vel.x = -particle.vel.x * 0.5;
            }
            if particle.pos.x >= grid_size {
                particle.pos.x = grid_size - 1.0;
                particle.vel.x = -particle.vel.x * 0.5;
            }
            if particle.pos.y < 0.0 {
                particle.pos.y = 0.0;
                particle.vel.y = -particle.vel.y * 0.5;
            }
            if particle.pos.y >= grid_size {
                particle.pos.y = grid_size - 1.0;
                particle.vel.y = -particle.vel.y * 0.5;
            }
        }
    }

    /// Set vibration mode
    pub fn set_mode(&mut self, m: u32, n: u32) {
        self.current_mode = PlateMode::new(m, n);
        self.reset_particles();
    }

    /// Reset particle positions
    pub fn reset_particles(&mut self) {
        self.particles = Self::spawn_particles(self.config.grid_size, self.particles.len());
    }
}

/// WASM entry point
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    web_sys::console::log_1(&"Chladni simulation initialized".into());
    Ok(())
}
