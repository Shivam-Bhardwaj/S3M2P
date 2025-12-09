//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: lib.rs | DNA/src/lib.rs
//! PURPOSE: Foundation library root - physics, math, world, data structures
//! MODIFIED: 2025-12-09
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════

// Suppress wasm_bindgen cfg warnings from macro expansion
#![allow(unexpected_cfgs)]

use glam::Vec2;
use rand::Rng;

// ============================================================================
// NEW ARCHITECTURE MODULES (DNA/CORE/SRC Refactor)
// See REFACTOR.md for full architecture documentation
// ============================================================================

/// WORLD - The Stage (coordinate systems, topology, grids)
pub mod world;

/// PHYSICS - The Rules (mechanics, fields, solvers)
pub mod physics;

/// MATH - The Language (vectors, matrices, complex numbers)
pub mod math;

/// DATA - Data Structures (arena, spatial grid, mesh, graph)
pub mod data;

// ============================================================================
// SIMULATION PRIMITIVES
// ============================================================================

// Note: coordinates module moved to world::transforms::astronomical
// Re-export for backward compatibility
pub use world::transforms::astronomical as coordinates;

pub mod sim;

/// Spatial indexing for cube-sphere LOD data (stars, planets, etc.)
pub mod spatial;
pub use spatial::*;

// ============================================================================
// SHARED UTILITY MODULES (Used by too.foo, helios, future projects)
// ============================================================================

/// Zone and exclusion area utilities
pub mod zones;
pub use zones::*;

/// Entity interaction effects
pub mod interaction;
pub use interaction::*;

// Note: random module moved to math::random
// Re-export for backward compatibility (don't glob re-export to avoid rand collision)
pub use math::random::{
    random_angle, random_direction, random_in_annulus, random_in_circle,
    random_in_rect, random_in_rect_with_margin, random_index, random_velocity,
    random_velocity_range, roll_chance,
};

/// Population statistics and metrics
pub mod statistics;
pub use statistics::*;

/// Color management and theme utilities
pub mod color;
pub use color::*;

/// Wave field simulation with FFT
/// Note: FFT migrated to physics/solvers/pde/spectral, Chladni to physics/fields/wave
pub mod wave_field;
pub use wave_field::*;

// Also export from new locations
pub use physics::solvers::pde::FFT2D;
pub use physics::fields::wave::{ChladniMode, PlateMode, WaveSimulation};

/// PLL (Phase-Locked Loop) circuit design
pub mod pll;
pub use pll::*;

/// SPICE circuit simulation engine
/// DEPRECATED: Use `physics::electromagnetics::lumped` or `spice_engine` crate
#[deprecated(since = "0.1.0", note = "use `physics::electromagnetics::lumped` instead")]
pub mod spice;

/// Autocrate crate generation algorithms
pub mod autocrate;

// ============================================================================
// MATH AND FILTERING MODULES
// ============================================================================

// Mat2 canonical location
pub use math::mat::Mat2;

/// Extended Kalman Filter for state estimation
/// DEPRECATED: Use `physics::solvers::filters::EKF`
#[deprecated(since = "0.1.0", note = "use `physics::solvers::filters::EKF` instead")]
pub mod ekf;

// Canonical export from new location
pub use physics::solvers::filters::{smooth_trajectory, EKF};

/// A* and grid-based pathfinding
pub mod pathfinding;
pub use pathfinding::{astar, GridMap, Heuristic, PathResult};

/// Export module (PDF, Gerber X2)
pub mod export;

/// CAD module (B-Rep solid modeling)
pub mod cad;

// ============================================================================
// CORE TYPES
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct Obstacle {
    pub center: Vec2,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoidRole {
    Herbivore,
    Carnivore,
    Scavenger,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoidState {
    Wander,
    Forage,
    Hunt,
    Flee,
    Reproduce,
    Dead, // Persist for scavenging
}

#[derive(Clone, Copy, Debug)]
pub struct Genome {
    pub role: BoidRole,
    pub max_speed: f32,     // 2.0 - 6.0
    pub agility: f32,       // Turn rate / Force multiplier (0.5 - 2.0)
    pub size: f32,          // 0.5 - 2.0 multiplier
    pub strength: f32,      // Combat/Health (0.5 - 2.0)
    pub sensor_radius: f32, // Vision (40.0 - 120.0)
    pub metabolism: f32,    // Energy cost (0.7 - 1.3)
    pub color_hs: (u16, u8),
}

impl Default for Genome {
    fn default() -> Self {
        Self {
            role: BoidRole::Herbivore,
            max_speed: 3.0,
            agility: 1.0,
            size: 1.0,
            strength: 1.0,
            sensor_radius: 60.0,
            metabolism: 1.0,
            color_hs: (120, 70),
        }
    }
}

impl Genome {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let role_roll = rng.gen::<f32>();
        let role = if role_roll < 0.6 {
            BoidRole::Herbivore
        } else if role_roll < 0.7 {
            BoidRole::Carnivore
        } else {
            BoidRole::Scavenger
        };

        let max_speed = rng.gen_range(2.0..=4.0);
        let agility = rng.gen_range(0.5..=2.0);
        let size = rng.gen_range(0.5..=2.0);
        let strength = rng.gen_range(0.5..=2.0);
        let sensor_radius = rng.gen_range(40.0..=120.0);
        let metabolism = rng.gen_range(0.7..=1.3);

        let color_hs = Self::compute_color_hs(role, max_speed, metabolism);

        Self {
            role,
            max_speed,
            agility,
            size,
            strength,
            sensor_radius,
            metabolism,
            color_hs,
        }
    }

    /// Compute color based on role, speed, and metabolism
    #[inline]
    fn compute_color_hs(role: BoidRole, max_speed: f32, metabolism: f32) -> (u16, u8) {
        let (base_hue, base_sat) = match role {
            BoidRole::Herbivore => (120, 70), // Green
            BoidRole::Carnivore => (0, 80),   // Red
            BoidRole::Scavenger => (280, 60), // Purple
        };

        let speed_norm = ((max_speed - 2.0) / 2.0).clamp(0.0, 1.0);
        let hue = (base_hue as f32 + speed_norm * 30.0) as u16 % 360;
        let sat = (base_sat as f32 + (metabolism - 0.7) * 20.0) as u8;
        (hue, sat.clamp(50, 100))
    }

    /// Hue from speed (blue=slow, red=fast), saturation from metabolism
    #[inline]
    pub fn color_hs(&self) -> (u16, u8) {
        self.color_hs
    }

    /// Mutate genome with one of 5 evolutionary events
    pub fn mutate(&self) -> Self {
        let mut rng = rand::thread_rng();
        let event_roll = rng.gen::<f32>();

        let mut new_genome = *self;

        // 5 Evolutionary Events
        if event_roll < 0.2 {
            // 1. Gigantism: ++Size/Strength, --Speed/Efficiency
            new_genome.size = (self.size * 1.2).clamp(0.5, 2.0);
            new_genome.strength = (self.strength * 1.2).clamp(0.5, 2.0);
            new_genome.max_speed = (self.max_speed * 0.9).clamp(2.0, 6.0);
            new_genome.metabolism = (self.metabolism * 1.1).clamp(0.7, 1.3);
        } else if event_roll < 0.4 {
            // 2. Miniaturization: --Size, ++Agility/Efficiency
            new_genome.size = (self.size * 0.8).clamp(0.5, 2.0);
            new_genome.agility = (self.agility * 1.2).clamp(0.5, 2.0);
            new_genome.metabolism = (self.metabolism * 0.9).clamp(0.7, 1.3);
        } else if event_roll < 0.6 {
            // 3. Swiftness: ++Speed, --Strength
            new_genome.max_speed = (self.max_speed * 1.2).clamp(2.0, 6.0);
            new_genome.strength = (self.strength * 0.9).clamp(0.5, 2.0);
        } else if event_roll < 0.8 {
            // 4. Hyper-Sense: ++Sensor Radius, --Efficiency
            new_genome.sensor_radius = (self.sensor_radius * 1.3).clamp(40.0, 120.0);
            new_genome.metabolism = (self.metabolism * 1.1).clamp(0.7, 1.3);
        } else if event_roll < 0.81 {
            // 5. Speciation: 1% chance to switch Role
            new_genome.role = match self.role {
                BoidRole::Herbivore => BoidRole::Carnivore,
                BoidRole::Carnivore => BoidRole::Scavenger,
                BoidRole::Scavenger => BoidRole::Herbivore,
            };
        } else {
            // Standard small mutations (19% chance)
            new_genome.max_speed = (self.max_speed * rng.gen_range(0.95..=1.05)).clamp(2.0, 6.0);
            new_genome.agility = (self.agility * rng.gen_range(0.95..=1.05)).clamp(0.5, 2.0);
            new_genome.size = (self.size * rng.gen_range(0.95..=1.05)).clamp(0.5, 2.0);
            new_genome.strength = (self.strength * rng.gen_range(0.95..=1.05)).clamp(0.5, 2.0);
            new_genome.sensor_radius =
                (self.sensor_radius * rng.gen_range(0.95..=1.05)).clamp(40.0, 120.0);
            new_genome.metabolism = (self.metabolism * rng.gen_range(0.95..=1.05)).clamp(0.7, 1.3);
        }

        // Recompute color
        new_genome.color_hs =
            Self::compute_color_hs(new_genome.role, new_genome.max_speed, new_genome.metabolism);

        new_genome
    }
}

// ============================================================================
// BOID ARENA - Fixed capacity, O(1) alloc/free, zero heap allocations
// ============================================================================

/// Generational index - catches use-after-free bugs at zero runtime cost
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoidHandle {
    index: u16,
    generation: u16,
}

impl BoidHandle {
    pub const INVALID: Self = Self {
        index: u16::MAX,
        generation: 0,
    };

    #[inline]
    pub fn is_valid(&self) -> bool {
        self.index != u16::MAX
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.index as usize
    }
}

/// Structure of Arrays layout for cache-friendly iteration
/// All arrays are the same length (CAPACITY)
pub struct BoidArena<const CAPACITY: usize> {
    // SoA layout - each field in its own array for cache locality
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub genes: Vec<Genome>,
    pub roles: Vec<BoidRole>,
    pub states: Vec<BoidState>,
    pub energy: Vec<f32>,
    pub age: Vec<f32>,
    pub generation: Vec<u16>,

    // Metadata
    pub alive: Vec<bool>,
    gen: Vec<u16>, // Generation counter for handles

    // Free list (indices of dead slots)
    free_list: Vec<u16>,
    free_count: usize,

    // Active count for fast iteration
    pub alive_count: usize,

    // Pre-allocated scratch buffers (avoid per-frame allocations)
    pub scratch_accel: Vec<Vec2>,
    pub scratch_density: Vec<u8>,
}

impl<const CAPACITY: usize> BoidArena<CAPACITY> {
    pub fn new() -> Self {
        let mut arena = Self {
            positions: vec![Vec2::ZERO; CAPACITY],
            velocities: vec![Vec2::ZERO; CAPACITY],
            genes: vec![Genome::default(); CAPACITY],
            roles: vec![BoidRole::Herbivore; CAPACITY],
            states: vec![BoidState::Wander; CAPACITY],
            energy: vec![0.0; CAPACITY],
            age: vec![0.0; CAPACITY],
            generation: vec![0; CAPACITY],
            alive: vec![false; CAPACITY],
            gen: vec![0; CAPACITY],
            free_list: vec![0; CAPACITY],
            free_count: CAPACITY,
            alive_count: 0,
            scratch_accel: vec![Vec2::ZERO; CAPACITY],
            scratch_density: vec![0; CAPACITY],
        };

        // Initialize free list (all slots available)
        for i in 0..CAPACITY {
            arena.free_list[i] = i as u16;
        }

        arena
    }

    /// Spawn a new boid, returns handle. O(1) operation.
    #[inline]
    pub fn spawn(&mut self, pos: Vec2, vel: Vec2, genes: Genome) -> BoidHandle {
        if self.free_count == 0 {
            return BoidHandle::INVALID;
        }

        self.free_count -= 1;
        let idx = self.free_list[self.free_count] as usize;

        self.positions[idx] = pos;
        self.velocities[idx] = vel;
        self.genes[idx] = genes;
        self.roles[idx] = genes.role;
        self.states[idx] = BoidState::Wander;
        self.energy[idx] = 100.0;
        self.age[idx] = 0.0;
        self.generation[idx] = 0;
        self.alive[idx] = true;
        self.gen[idx] = self.gen[idx].wrapping_add(1);
        self.alive_count += 1;

        BoidHandle {
            index: idx as u16,
            generation: self.gen[idx],
        }
    }

    /// Spawn with inherited traits (for reproduction)
    #[inline]
    pub fn spawn_child(&mut self, parent_idx: usize) -> BoidHandle {
        if self.free_count == 0 || !self.alive[parent_idx] {
            return BoidHandle::INVALID;
        }

        let mut rng = rand::thread_rng();
        let pos = self.positions[parent_idx];
        let vel = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0));
        let genes = self.genes[parent_idx].mutate();
        let parent_gen = self.generation[parent_idx];

        // Cost to parent
        self.energy[parent_idx] -= 50.0;

        self.free_count -= 1;
        let idx = self.free_list[self.free_count] as usize;

        self.positions[idx] = pos;
        self.velocities[idx] = vel;
        self.genes[idx] = genes;
        self.roles[idx] = genes.role;
        self.states[idx] = BoidState::Wander;
        self.energy[idx] = 80.0;
        self.age[idx] = 0.0;
        self.generation[idx] = parent_gen + 1;
        self.alive[idx] = true;
        self.gen[idx] = self.gen[idx].wrapping_add(1);
        self.alive_count += 1;

        BoidHandle {
            index: idx as u16,
            generation: self.gen[idx],
        }
    }

    /// Kill a boid, returns slot to free list. O(1) operation.
    /// Sets state to Dead for scavenging before cleanup
    #[inline]
    pub fn kill(&mut self, idx: usize) {
        if idx < CAPACITY && self.alive[idx] {
            self.states[idx] = BoidState::Dead;
            self.alive[idx] = false;
            self.free_list[self.free_count] = idx as u16;
            self.free_count += 1;
            self.alive_count -= 1;
        }
    }

    /// Check if handle is still valid
    #[inline]
    pub fn is_alive(&self, handle: BoidHandle) -> bool {
        let idx = handle.index as usize;
        idx < CAPACITY && self.alive[idx] && self.gen[idx] == handle.generation
    }

    /// Iterate over all alive boid indices
    #[inline]
    pub fn iter_alive(&self) -> impl Iterator<Item = usize> + '_ {
        (0..CAPACITY).filter(|&i| self.alive[i])
    }

    /// Get remaining capacity
    #[inline]
    pub fn remaining_capacity(&self) -> usize {
        self.free_count
    }
}

impl<const CAPACITY: usize> Default for BoidArena<CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SPATIAL GRID - Fixed capacity cells
// ============================================================================

/// Spatial grid with fixed-size cells (no heap allocation per cell)
pub struct SpatialGrid<const CELL_CAPACITY: usize> {
    cell_size: f32,
    cols: usize,
    rows: usize,
    // Each cell stores up to CELL_CAPACITY indices
    cells: Vec<[u16; CELL_CAPACITY]>,
    cell_counts: Vec<u8>,
}

impl<const CELL_CAPACITY: usize> SpatialGrid<CELL_CAPACITY> {
    pub fn new(width: f32, height: f32, cell_size: f32) -> Self {
        let cols = ((width / cell_size).ceil() as usize).max(1);
        let rows = ((height / cell_size).ceil() as usize).max(1);
        let cell_count = cols * rows;

        Self {
            cell_size,
            cols,
            rows,
            cells: vec![[0; CELL_CAPACITY]; cell_count],
            cell_counts: vec![0; cell_count],
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        let new_cols = ((width / self.cell_size).ceil() as usize).max(1);
        let new_rows = ((height / self.cell_size).ceil() as usize).max(1);

        if new_cols != self.cols || new_rows != self.rows {
            self.cols = new_cols;
            self.rows = new_rows;
            let cell_count = new_cols * new_rows;
            self.cells.resize(cell_count, [0; CELL_CAPACITY]);
            self.cell_counts.resize(cell_count, 0);
        }
    }

    #[inline]
    fn cell_index(&self, pos: Vec2) -> usize {
        let col = ((pos.x / self.cell_size) as usize).min(self.cols.saturating_sub(1));
        let row = ((pos.y / self.cell_size) as usize).min(self.rows.saturating_sub(1));
        row * self.cols + col
    }

    /// Clear all cells (O(cells) not O(boids))
    pub fn clear(&mut self) {
        for count in &mut self.cell_counts {
            *count = 0;
        }
    }

    /// Insert boid index into grid
    #[inline]
    pub fn insert(&mut self, idx: u16, pos: Vec2) {
        let cell_idx = self.cell_index(pos);
        let count = self.cell_counts[cell_idx] as usize;
        if count < CELL_CAPACITY {
            self.cells[cell_idx][count] = idx;
            self.cell_counts[cell_idx] += 1;
        }
    }

    /// Build grid from arena (only alive boids)
    pub fn build<const CAP: usize>(&mut self, arena: &BoidArena<CAP>) {
        self.clear();
        for idx in arena.iter_alive() {
            self.insert(idx as u16, arena.positions[idx]);
        }
    }

    /// Query neighbors, writes indices to output buffer, returns count
    pub fn query_neighbors<const CAP: usize>(
        &self,
        pos: Vec2,
        radius: f32,
        arena: &BoidArena<CAP>,
        exclude_idx: usize,
        output: &mut [u16],
    ) -> usize {
        let radius_sq = radius * radius;
        let mut count = 0;

        let min_col = ((pos.x - radius) / self.cell_size).floor().max(0.0) as usize;
        let max_col = (((pos.x + radius) / self.cell_size).ceil() as usize).min(self.cols);
        let min_row = ((pos.y - radius) / self.cell_size).floor().max(0.0) as usize;
        let max_row = (((pos.y + radius) / self.cell_size).ceil() as usize).min(self.rows);

        for row in min_row..max_row {
            for col in min_col..max_col {
                let cell_idx = row * self.cols + col;
                let cell_count = self.cell_counts[cell_idx] as usize;

                for i in 0..cell_count {
                    let other_idx = self.cells[cell_idx][i] as usize;
                    if other_idx == exclude_idx {
                        continue;
                    }

                    let dist_sq = (arena.positions[other_idx] - pos).length_squared();
                    if dist_sq < radius_sq && count < output.len() {
                        output[count] = other_idx as u16;
                        count += 1;
                    }
                }
            }
        }

        count
    }

    /// Count neighbors (no allocation)
    #[inline]
    pub fn count_neighbors<const CAP: usize>(
        &self,
        pos: Vec2,
        radius: f32,
        arena: &BoidArena<CAP>,
        exclude_idx: usize,
    ) -> usize {
        let mut neighbors = [0u16; 64];
        self.query_neighbors(pos, radius, arena, exclude_idx, &mut neighbors)
    }
}

// ============================================================================
// FLOCKING FORCES - Zero allocation
// ============================================================================

/// Calculate flocking forces using spatial grid, writes to arena's scratch buffer
/// Now state-aware: different behaviors for Hunt, Flee, Forage, etc.
pub fn compute_flocking_forces<const CAP: usize, const CELL_CAP: usize>(
    arena: &mut BoidArena<CAP>,
    grid: &SpatialGrid<CELL_CAP>,
    vision_radius: f32,
    obstacles: &[Obstacle],
) {
    let mut neighbors = [0u16; 64]; // Stack-allocated neighbor buffer

    for idx in 0..CAP {
        if !arena.alive[idx] {
            arena.scratch_accel[idx] = Vec2::ZERO;
            continue;
        }

        let pos = arena.positions[idx];
        let state = arena.states[idx];
        let role = arena.roles[idx];
        let sensor_radius = arena.genes[idx].sensor_radius;
        let effective_radius = sensor_radius.max(vision_radius);

        let neighbor_count =
            grid.query_neighbors(pos, effective_radius, arena, idx, &mut neighbors);

        // Store density for population dynamics
        arena.scratch_density[idx] = neighbor_count.min(255) as u8;

        let mut force = Vec2::ZERO;

        match state {
            BoidState::Flee => {
                // Flee from predators
                let mut flee_force = Vec2::ZERO;
                for &neighbor_idx in neighbors.iter().take(neighbor_count) {
                    let other_idx = neighbor_idx as usize;
                    if arena.roles[other_idx] == BoidRole::Carnivore && arena.alive[other_idx] {
                        let diff = pos - arena.positions[other_idx];
                        let dist = diff.length();
                        if dist > 0.001 {
                            flee_force += (diff / dist) * (100.0 / dist);
                        }
                    }
                }
                force += flee_force * 3.0; // Strong flee response
            }
            BoidState::Hunt => {
                // Seek nearest prey
                let mut seek_force = Vec2::ZERO;
                let mut closest_dist = f32::MAX;
                let mut closest_prey = None;

                for &neighbor_idx in neighbors.iter().take(neighbor_count) {
                    let other_idx = neighbor_idx as usize;
                    if arena.roles[other_idx] != BoidRole::Carnivore && arena.alive[other_idx] {
                        let diff = arena.positions[other_idx] - pos;
                        let dist = diff.length();
                        if dist < closest_dist {
                            closest_dist = dist;
                            closest_prey = Some(other_idx);
                        }
                    }
                }

                if let Some(prey_idx) = closest_prey {
                    let diff = arena.positions[prey_idx] - pos;
                    let dist = diff.length();
                    if dist > 0.001 {
                        seek_force = diff / dist * 2.0;
                    }
                }
                force += seek_force;
            }
            BoidState::Forage => {
                // Wander with slight cohesion to same-species
                if neighbor_count > 0 {
                    let mut cohesion = Vec2::ZERO;
                    let mut separation = Vec2::ZERO;
                    let mut same_species_count = 0;

                    for &neighbor_idx in neighbors.iter().take(neighbor_count) {
                        let other_idx = neighbor_idx as usize;
                        if arena.roles[other_idx] == role && arena.alive[other_idx] {
                            let diff = arena.positions[other_idx] - pos;
                            let dist = diff.length();
                            cohesion += arena.positions[other_idx];
                            if dist > 0.001 && dist < 30.0 {
                                separation -= diff / dist;
                            }
                            same_species_count += 1;
                        }
                    }

                    if same_species_count > 0 {
                        cohesion = (cohesion / same_species_count as f32) - pos;
                        force += cohesion * 0.5 + separation * 2.0;
                    }
                }
            }
            BoidState::Reproduce => {
                // Seek same-species mates
                if neighbor_count > 0 {
                    let mut mate_force = Vec2::ZERO;
                    for &neighbor_idx in neighbors.iter().take(neighbor_count) {
                        let other_idx = neighbor_idx as usize;
                        if arena.roles[other_idx] == role
                            && arena.alive[other_idx]
                            && arena.states[other_idx] == BoidState::Reproduce
                        {
                            let diff = arena.positions[other_idx] - pos;
                            let dist = diff.length();
                            if dist > 0.001 {
                                mate_force += diff / dist * 1.5;
                            }
                        }
                    }
                    force += mate_force;
                }
            }
            BoidState::Wander | BoidState::Dead => {
                // Standard flocking behavior
                if neighbor_count > 0 {
                    let mut cohesion = Vec2::ZERO;
                    let mut alignment = Vec2::ZERO;
                    let mut separation = Vec2::ZERO;
                    let mut same_species_count = 0;

                    for &neighbor_idx in neighbors.iter().take(neighbor_count) {
                        let other_idx = neighbor_idx as usize;
                        if arena.roles[other_idx] == role && arena.alive[other_idx] {
                            let diff = arena.positions[other_idx] - pos;
                            let dist = diff.length();

                            cohesion += arena.positions[other_idx];
                            alignment += arena.velocities[other_idx];
                            if dist > 0.001 {
                                // Inverse square law for strong close-range repulsion
                                let repulsion_strength = (20.0 / dist).powi(2).min(50.0);
                                separation -= (diff / dist) * repulsion_strength;
                            }
                            same_species_count += 1;
                        }
                    }

                    if same_species_count > 0 {
                        let n = same_species_count as f32;
                        cohesion = (cohesion / n - pos) * 0.8; // Reduced cohesion
                        alignment /= n;
                        separation /= n;
                        force += cohesion + alignment * 1.0 + separation * 2.5; // Increased separation weight
                    }
                }
            }
        }

        // Apply agility multiplier
        let agility_mult = arena.genes[idx].agility;
        force *= agility_mult;

        // Always avoid obstacles
        let avoidance = compute_obstacle_avoidance(pos, obstacles);
        force += avoidance;

        arena.scratch_accel[idx] = force;
    }
}

#[inline]
fn compute_obstacle_avoidance(pos: Vec2, obstacles: &[Obstacle]) -> Vec2 {
    let mut force = Vec2::ZERO;
    const BUFFER: f32 = 50.0;

    for obs in obstacles {
        let diff = pos - obs.center;
        let d = diff.length();
        if d < obs.radius + BUFFER && d > 0.001 {
            let repulsion = diff / d; // Safe normalization
            force += repulsion * (100.0 / d);
        }
    }
    force
}

// ============================================================================
// STATE MACHINE - Update boid states based on environment
// ============================================================================

/// Update states for all alive boids based on energy, neighbors, and threats
pub fn update_states<const CAP: usize, const CELL_CAP: usize>(
    arena: &mut BoidArena<CAP>,
    grid: &SpatialGrid<CELL_CAP>,
) {
    let mut neighbors = [0u16; 64];

    for idx in 0..CAP {
        if !arena.alive[idx] {
            continue;
        }

        let pos = arena.positions[idx];
        let role = arena.roles[idx];
        let energy = arena.energy[idx];
        let sensor_radius = arena.genes[idx].sensor_radius;

        // Check for predators (Carnivores) in sensor range
        let mut has_predator = false;
        let neighbor_count = grid.query_neighbors(pos, sensor_radius, arena, idx, &mut neighbors);

        for &neighbor_idx in neighbors.iter().take(neighbor_count) {
            let other_idx = neighbor_idx as usize;
            if arena.roles[other_idx] == BoidRole::Carnivore && role != BoidRole::Carnivore {
                has_predator = true;
                break;
            }
        }

        // State transition logic
        if has_predator {
            arena.states[idx] = BoidState::Flee;
        } else if role == BoidRole::Carnivore {
            // Check for prey
            let mut has_prey = false;
            for &neighbor_idx in neighbors.iter().take(neighbor_count) {
                let other_idx = neighbor_idx as usize;
                if arena.roles[other_idx] != BoidRole::Carnivore && arena.alive[other_idx] {
                    has_prey = true;
                    break;
                }
            }
            if has_prey {
                arena.states[idx] = BoidState::Hunt;
            } else if energy > 160.0 {
                arena.states[idx] = BoidState::Reproduce;
            } else {
                arena.states[idx] = BoidState::Wander;
            }
        } else if energy < 80.0 {
            // Low energy -> Forage
            arena.states[idx] = BoidState::Forage;
        } else if energy > 160.0 {
            arena.states[idx] = BoidState::Reproduce;
        } else {
            arena.states[idx] = BoidState::Wander;
        }
    }
}

// ============================================================================
// PREDATION & FEEDING - Handle interactions between boids
// ============================================================================

/// Process predation: Carnivores attack and consume other boids
pub fn process_predation<const CAP: usize>(arena: &mut BoidArena<CAP>) {
    const PREDATION_RADIUS: f32 = 15.0; // Close contact required
    const PREDATION_DAMAGE: f32 = 20.0;
    const PREDATION_GAIN: f32 = 30.0;

    for idx in 0..CAP {
        if !arena.alive[idx] || arena.roles[idx] != BoidRole::Carnivore {
            continue;
        }

        let pos = arena.positions[idx];

        // Find nearby prey
        for other_idx in 0..CAP {
            if !arena.alive[other_idx] || other_idx == idx {
                continue;
            }

            // Carnivores don't eat other carnivores
            if arena.roles[other_idx] == BoidRole::Carnivore {
                continue;
            }

            let other_pos = arena.positions[other_idx];
            let dist_sq = pos.distance_squared(other_pos);

            if dist_sq < PREDATION_RADIUS * PREDATION_RADIUS {
                // Attack!
                let damage = PREDATION_DAMAGE * arena.genes[idx].strength;
                arena.energy[other_idx] -= damage;

                // Carnivore gains energy
                arena.energy[idx] = (arena.energy[idx] + PREDATION_GAIN).min(200.0);

                // Kill prey if energy depleted
                if arena.energy[other_idx] <= 0.0 {
                    arena.kill(other_idx);
                }
            }
        }
    }
}

/// Process scavenging: Scavengers consume dead boids
pub fn process_scavenging<const CAP: usize>(arena: &mut BoidArena<CAP>) {
    const SCAVENGING_RADIUS: f32 = 20.0;
    const SCAVENGING_GAIN: f32 = 15.0;

    for idx in 0..CAP {
        if !arena.alive[idx] || arena.roles[idx] != BoidRole::Scavenger {
            continue;
        }

        let pos = arena.positions[idx];

        // Find dead boids nearby
        for other_idx in 0..CAP {
            if arena.alive[other_idx] || arena.states[other_idx] != BoidState::Dead {
                continue;
            }

            let other_pos = arena.positions[other_idx];
            let dist_sq = pos.distance_squared(other_pos);

            if dist_sq < SCAVENGING_RADIUS * SCAVENGING_RADIUS {
                // Consume corpse
                arena.energy[idx] = (arena.energy[idx] + SCAVENGING_GAIN).min(200.0);
                // Remove corpse (fully free the slot)
                arena.states[other_idx] = BoidState::Wander;
            }
        }
    }
}

// ============================================================================
// SIMULATION STEP - Main update loop
// ============================================================================

pub struct SimConfig {
    pub carrying_capacity: usize,
    pub reproduction_threshold: f32,
    pub reproduction_cost: f32,
    pub max_age: f32,
    pub base_mortality: f32,
    pub starvation_threshold: f32,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            carrying_capacity: 800,
            reproduction_threshold: 120.0, // Easier to reproduce
            reproduction_cost: 40.0,       // Cheaper reproduction
            max_age: 2000.0,               // Longer lifespan
            base_mortality: 0.00002,       // Much gentler base mortality
            starvation_threshold: 10.0,    // Only die when very low energy
        }
    }
}

/// Single simulation step - zero heap allocations
/// Now integrates state machine, predation, and scavenging
pub fn simulation_step<const CAP: usize, const CELL_CAP: usize>(
    arena: &mut BoidArena<CAP>,
    grid: &SpatialGrid<CELL_CAP>,
    config: &SimConfig,
    width: f32,
    height: f32,
    dt: f32,
) -> (usize, usize) {
    // returns (births, deaths)
    let mut rng = rand::thread_rng();
    let mut births = 0usize;
    let mut deaths = 0usize;
    let population = arena.alive_count;

    // Phase 0: Update states based on environment
    update_states(arena, grid);

    // Collect reproduction candidates first (to avoid borrowing issues)
    let mut reproduce_indices = [0u16; 128];
    let mut reproduce_count = 0;

    // Phase 1: Apply forces and update physics
    for idx in 0..CAP {
        if !arena.alive[idx] {
            continue;
        }

        // Apply acceleration
        let accel = arena.scratch_accel[idx] * 0.05;
        arena.velocities[idx] += accel;

        // Limit speed (guard against zero division)
        let max_speed = arena.genes[idx].max_speed;
        let speed = arena.velocities[idx].length();
        if speed > max_speed && speed > 0.0001 {
            arena.velocities[idx] = arena.velocities[idx] / speed * max_speed;
        }

        // Update position
        arena.positions[idx] += arena.velocities[idx] * dt;

        // Wrap around
        if arena.positions[idx].x < 0.0 {
            arena.positions[idx].x += width;
        }
        if arena.positions[idx].x >= width {
            arena.positions[idx].x -= width;
        }
        if arena.positions[idx].y < 0.0 {
            arena.positions[idx].y += height;
        }
        if arena.positions[idx].y >= height {
            arena.positions[idx].y -= height;
        }

        // Metabolism (size affects cost)
        let size_cost = arena.genes[idx].size;
        let metabolism_cost = speed * 0.002 * arena.genes[idx].metabolism * size_cost;
        arena.energy[idx] -= metabolism_cost;

        // Aging
        arena.age[idx] += dt;

        // Check reproduction (only in Reproduce state and high energy)
        if arena.states[idx] == BoidState::Reproduce
            && arena.energy[idx] > config.reproduction_threshold
            && reproduce_count < 128
            && population + reproduce_count < config.carrying_capacity
        {
            reproduce_indices[reproduce_count] = idx as u16;
            reproduce_count += 1;
        }
    }

    // Phase 2: Process interactions (predation, scavenging)
    process_predation(arena);
    process_scavenging(arena);

    // Phase 3: Reproduction (separate pass to avoid borrow conflicts)
    for &parent_idx_u16 in reproduce_indices.iter().take(reproduce_count) {
        let parent_idx = parent_idx_u16 as usize;
        if arena.alive[parent_idx] && arena.energy[parent_idx] > config.reproduction_threshold {
            // Check for mate nearby
            let mut has_mate = false;
            let pos = arena.positions[parent_idx];
            let mut neighbors = [0u16; 64];
            let neighbor_count = grid.query_neighbors(pos, 30.0, arena, parent_idx, &mut neighbors);

            for &neighbor_idx in neighbors.iter().take(neighbor_count) {
                let other_idx = neighbor_idx as usize;
                if arena.alive[other_idx]
                    && arena.roles[other_idx] == arena.roles[parent_idx]
                    && arena.states[other_idx] == BoidState::Reproduce
                    && other_idx != parent_idx
                {
                    has_mate = true;
                    break;
                }
            }

            if has_mate {
                let handle = arena.spawn_child(parent_idx);
                if handle.is_valid() {
                    births += 1;
                }
            }
        }
    }

    // Phase 4: Death checks
    for idx in 0..CAP {
        if !arena.alive[idx] {
            continue;
        }

        let should_die =
            // Starvation - primary death cause
            arena.energy[idx] <= config.starvation_threshold ||
            // Old age guaranteed death
            arena.age[idx] > config.max_age ||
            // Carrying capacity pressure only when significantly over
            {
                let over_capacity = population > config.carrying_capacity;
                let pop_excess = if over_capacity {
                    (population - config.carrying_capacity) as f32 / config.carrying_capacity as f32
                } else { 0.0 };

                // Only apply population pressure, age mortality is gentle
                let death_prob = config.base_mortality + pop_excess * 0.02;

                rng.gen::<f32>() < death_prob
            };

        if should_die {
            arena.kill(idx);
            deaths += 1;
        }
    }

    (births, deaths)
}

// ============================================================================
// FOOD SOURCES
// ============================================================================

#[derive(Clone, Debug)]
pub struct FoodSource {
    pub position: Vec2,
    pub energy: f32,
    pub max_energy: f32,
    pub radius: f32,
    pub regen_rate: f32,
    pub depleted_timer: f32, // Time since last depletion
}

impl FoodSource {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            energy: 600.0,
            max_energy: 800.0,
            radius: 90.0,
            regen_rate: 5.0,
            depleted_timer: 0.0,
        }
    }

    #[inline]
    pub fn consume(&mut self, amount: f32) -> f32 {
        let taken = amount.min(self.energy);
        self.energy -= taken;
        if taken > 0.0 {
            self.depleted_timer = 0.0;
        }
        taken
    }

    #[inline]
    pub fn regenerate(&mut self, dt: f32, season_multiplier: f32) {
        self.depleted_timer += dt;
        // Faster regen when not being consumed, affected by season
        let regen = self.regen_rate * season_multiplier * dt;
        self.energy = (self.energy + regen).min(self.max_energy);
    }

    #[inline]
    pub fn is_depleted(&self) -> bool {
        self.energy < self.max_energy * 0.1
    }

    #[inline]
    pub fn fullness(&self) -> f32 {
        self.energy / self.max_energy
    }
}

/// Predator zone - dangerous area that drains energy
#[derive(Clone, Debug)]
pub struct PredatorZone {
    pub position: Vec2,
    pub radius: f32,
    pub intensity: f32,
    pub active: bool,
    pub lifetime: f32,
}

impl PredatorZone {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            radius: 120.0,
            intensity: 3.0,
            active: true,
            lifetime: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.lifetime += dt;
        // Predators fade after ~15 seconds
        if self.lifetime > 900.0 {
            self.active = false;
        }
        // Intensity pulses
        self.intensity = 2.0 + (self.lifetime * 0.1).sin() * 1.5;
    }
}

/// World events for drama
#[derive(Clone, Debug)]
pub enum WorldEvent {
    Famine,        // Food stops regenerating
    Bloom,         // Food regenerates 3x faster
    PredatorSpawn, // New predator zone appears
    Migration,     // Boids get pushed in a direction
    Earthquake,    // Randomize all velocities
}

/// Seasonal cycle affects food and mortality
#[derive(Clone, Copy, Debug)]
pub struct SeasonCycle {
    pub time: f32,
    pub period: f32, // ~30 seconds per season
}

impl Default for SeasonCycle {
    fn default() -> Self {
        Self::new()
    }
}

impl SeasonCycle {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            period: 1800.0,
        } // 30 second seasons
    }

    pub fn update(&mut self, dt: f32) {
        self.time += dt;
    }

    /// Returns 0.0-1.0 season phase (0=winter, 0.5=summer)
    #[inline]
    pub fn phase(&self) -> f32 {
        (self.time / self.period).fract()
    }

    /// Food multiplier: low in winter, high in summer
    #[inline]
    pub fn food_multiplier(&self) -> f32 {
        let phase = self.phase();
        // Sinusoidal: 0.3 in winter, 2.0 in summer
        0.3 + 1.7 * (phase * std::f32::consts::TAU).sin().max(0.0)
    }

    /// Returns season name
    pub fn season_name(&self) -> &'static str {
        let phase = self.phase();
        if phase < 0.25 {
            "SPRING"
        } else if phase < 0.5 {
            "SUMMER"
        } else if phase < 0.75 {
            "AUTUMN"
        } else {
            "WINTER"
        }
    }
}

/// Feed boids from food sources - zero allocations
pub fn feed_from_sources<const CAP: usize>(
    arena: &mut BoidArena<CAP>,
    food_sources: &mut [FoodSource],
    season: &SeasonCycle,
) {
    let food_mult = season.food_multiplier();

    for idx in 0..CAP {
        if !arena.alive[idx] {
            continue;
        }

        let pos = arena.positions[idx];

        for food in food_sources.iter_mut() {
            let dist = pos.distance(food.position);
            if dist < food.radius && food.energy > 0.0 {
                // More food in summer, less in winter
                let consumed = food.consume(0.8 + food_mult * 0.4);
                arena.energy[idx] = (arena.energy[idx] + consumed).min(200.0);
                break;
            }
        }
    }

    // Regenerate food based on season
    for food in food_sources.iter_mut() {
        food.regenerate(1.0, food_mult);
    }
}

/// Apply predator zone damage to boids
pub fn apply_predator_zones<const CAP: usize>(
    arena: &mut BoidArena<CAP>,
    predators: &[PredatorZone],
) -> usize {
    let mut kills = 0;

    for idx in 0..CAP {
        if !arena.alive[idx] {
            continue;
        }

        let pos = arena.positions[idx];

        for pred in predators {
            if !pred.active {
                continue;
            }

            let dist = pos.distance(pred.position);
            if dist < pred.radius {
                // Drain energy based on proximity
                let damage = pred.intensity * (1.0 - dist / pred.radius);
                arena.energy[idx] -= damage;

                // Push boids away from predator
                if dist > 1.0 {
                    let flee = (pos - pred.position).normalize() * 2.0;
                    arena.velocities[idx] += flee;
                }

                if arena.energy[idx] <= 0.0 {
                    kills += 1;
                }
            }
        }
    }

    kills
}

/// Trigger a migration event - push all boids in a direction
pub fn trigger_migration<const CAP: usize>(
    arena: &mut BoidArena<CAP>,
    direction: Vec2,
    strength: f32,
) {
    for idx in 0..CAP {
        if arena.alive[idx] {
            arena.velocities[idx] += direction * strength;
        }
    }
}

/// Trigger earthquake - randomize velocities
pub fn trigger_earthquake<const CAP: usize>(arena: &mut BoidArena<CAP>) {
    let mut rng = rand::thread_rng();
    use rand::Rng;

    for idx in 0..CAP {
        if arena.alive[idx] {
            arena.velocities[idx] = Vec2::new(rng.gen_range(-3.0..3.0), rng.gen_range(-3.0..3.0));
            // Stress from earthquake
            arena.energy[idx] -= 5.0;
        }
    }
}

// ============================================================================
// DIVERSITY & ECOSYSTEM HEALTH
// ============================================================================

/// Compute ecosystem diversity score (0.0 = monoculture, 1.0 = highly diverse)
/// Based on role distribution and trait variance
pub fn compute_diversity<const CAP: usize>(arena: &BoidArena<CAP>) -> f32 {
    if arena.alive_count < 10 {
        return 1.0; // Too few to measure, assume diverse
    }

    let mut herbivore_count = 0usize;
    let mut carnivore_count = 0usize;
    let mut scavenger_count = 0usize;
    let mut speed_sum = 0.0f32;
    let mut speed_sq_sum = 0.0f32;

    for idx in arena.iter_alive() {
        match arena.roles[idx] {
            BoidRole::Herbivore => herbivore_count += 1,
            BoidRole::Carnivore => carnivore_count += 1,
            BoidRole::Scavenger => scavenger_count += 1,
        }
        let speed = arena.genes[idx].max_speed;
        speed_sum += speed;
        speed_sq_sum += speed * speed;
    }

    let total = arena.alive_count as f32;

    // Role diversity: Shannon entropy normalized
    // Perfect balance = 0.33, 0.33, 0.33 -> entropy = log2(3) ≈ 1.58
    let h_frac = herbivore_count as f32 / total;
    let c_frac = carnivore_count as f32 / total;
    let s_frac = scavenger_count as f32 / total;

    let mut entropy = 0.0f32;
    if h_frac > 0.0 {
        entropy -= h_frac * h_frac.log2();
    }
    if c_frac > 0.0 {
        entropy -= c_frac * c_frac.log2();
    }
    if s_frac > 0.0 {
        entropy -= s_frac * s_frac.log2();
    }

    let max_entropy = 3.0f32.log2(); // ~1.58
    let role_diversity = (entropy / max_entropy).clamp(0.0, 1.0);

    // Trait diversity: coefficient of variation of speed
    let speed_mean = speed_sum / total;
    let speed_variance = (speed_sq_sum / total) - (speed_mean * speed_mean);
    let speed_std = speed_variance.max(0.0).sqrt();
    let cv = if speed_mean > 0.0 {
        speed_std / speed_mean
    } else {
        0.0
    };
    // CV of 0.3+ is healthy diversity, normalize
    let trait_diversity = (cv / 0.4).clamp(0.0, 1.0);

    // Combined score (weighted)
    0.7 * role_diversity + 0.3 * trait_diversity
}

/// Trigger mass extinction - kills most boids, resets ecosystem
pub fn trigger_mass_extinction<const CAP: usize>(
    arena: &mut BoidArena<CAP>,
    kill_fraction: f32,
    width: f32,
    height: f32,
) {
    let mut rng = rand::thread_rng();
    use rand::Rng;

    let mut killed = 0usize;
    let target_kills = (arena.alive_count as f32 * kill_fraction) as usize;

    for idx in 0..CAP {
        if !arena.alive[idx] {
            continue;
        }
        if killed >= target_kills {
            break;
        }

        // Random chance to survive (larger/stronger have slight advantage)
        let survival_bonus = arena.genes[idx].strength * 0.1;
        if rng.gen::<f32>() > survival_bonus {
            arena.kill(idx);
            killed += 1;
        }
    }

    // Spawn a few diverse founders to reseed (use actual world dimensions)
    let founders = 10.min(CAP - arena.alive_count);
    let margin = 50.0f32.min(width * 0.1).min(height * 0.1);
    for _ in 0..founders {
        let pos = Vec2::new(
            rng.gen_range(margin..(width - margin).max(margin + 1.0)),
            rng.gen_range(margin..(height - margin).max(margin + 1.0)),
        );
        let vel = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0));
        arena.spawn(pos, vel, Genome::random());
    }
}

// ============================================================================
// RENDERING HELPERS
// ============================================================================

/// Get color components for a boid (hue, saturation, lightness)
#[inline]
pub fn get_boid_color<const CAP: usize>(arena: &BoidArena<CAP>, idx: usize) -> (u16, u8, u8) {
    let (hue, sat) = arena.genes[idx].color_hs();
    let energy_norm = (arena.energy[idx] / 200.0).clamp(0.0, 1.0);
    let lightness = (25.0 + energy_norm * 55.0) as u8;
    (hue, sat, lightness)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_spawn_kill() {
        let mut arena: BoidArena<100> = BoidArena::new();
        assert_eq!(arena.alive_count, 0);
        assert_eq!(arena.remaining_capacity(), 100);

        let h1 = arena.spawn(Vec2::new(10.0, 10.0), Vec2::ZERO, Genome::random());
        assert!(h1.is_valid());
        assert_eq!(arena.alive_count, 1);

        arena.kill(h1.index as usize);
        assert_eq!(arena.alive_count, 0);
        assert!(!arena.is_alive(h1)); // Handle invalidated by generation bump
    }

    #[test]
    fn test_spatial_grid() {
        let mut arena: BoidArena<100> = BoidArena::new();
        let h1 = arena.spawn(Vec2::new(10.0, 10.0), Vec2::ZERO, Genome::random());
        let _h2 = arena.spawn(Vec2::new(15.0, 10.0), Vec2::ZERO, Genome::random());
        let _h3 = arena.spawn(Vec2::new(100.0, 100.0), Vec2::ZERO, Genome::random());

        let mut grid: SpatialGrid<16> = SpatialGrid::new(200.0, 200.0, 50.0);
        grid.build(&arena);

        let count = grid.count_neighbors(Vec2::new(10.0, 10.0), 20.0, &arena, h1.index as usize);
        assert_eq!(count, 1); // Should find boid at (15, 10) but not self
    }

    // ============================================================================
    // EVOLUTION TESTS
    // ============================================================================

    #[test]
    fn test_genome_defaults() {
        let genome = Genome::random();
        assert!(genome.max_speed >= 2.0 && genome.max_speed <= 4.0);
        assert!(genome.agility >= 0.5 && genome.agility <= 2.0);
        assert!(genome.size >= 0.5 && genome.size <= 2.0);
        assert!(genome.strength >= 0.5 && genome.strength <= 2.0);
        assert!(genome.sensor_radius >= 40.0 && genome.sensor_radius <= 120.0);
        assert!(genome.metabolism >= 0.7 && genome.metabolism <= 1.3);
    }

    #[test]
    fn test_mutation_gigantism() {
        let parent = Genome {
            role: BoidRole::Herbivore,
            max_speed: 3.0,
            agility: 1.0,
            size: 1.0,
            strength: 1.0,
            sensor_radius: 60.0,
            metabolism: 1.0,
            color_hs: (120, 70),
        };

        // Force gigantism by testing multiple mutations until we get one
        let mut found_gigantism = false;
        for _ in 0..100 {
            let child = parent.mutate();
            if child.size > parent.size * 1.15 && child.strength > parent.strength * 1.15 {
                assert!(
                    child.max_speed < parent.max_speed * 1.05
                        || child.max_speed <= parent.max_speed
                );
                found_gigantism = true;
                break;
            }
        }
        assert!(found_gigantism, "Should find gigantism mutation");
    }

    #[test]
    fn test_role_inheritance() {
        let parent = Genome {
            role: BoidRole::Herbivore,
            max_speed: 3.0,
            agility: 1.0,
            size: 1.0,
            strength: 1.0,
            sensor_radius: 60.0,
            metabolism: 1.0,
            color_hs: (120, 70),
        };

        let mut role_changes = 0;
        for _ in 0..100 {
            let child = parent.mutate();
            if child.role != parent.role {
                role_changes += 1;
            }
        }

        // Speciation should be rare (~1%)
        assert!(
            role_changes <= 5,
            "Role mutation rate should be low, got {} changes",
            role_changes
        );
    }

    // ============================================================================
    // STATE MACHINE TESTS
    // ============================================================================

    #[test]
    fn test_state_transition_flee() {
        let mut arena: BoidArena<100> = BoidArena::new();
        let mut grid: SpatialGrid<16> = SpatialGrid::new(200.0, 200.0, 60.0);

        // Spawn a Herbivore
        let herb_genes = Genome {
            role: BoidRole::Herbivore,
            max_speed: 3.0,
            agility: 1.0,
            size: 1.0,
            strength: 1.0,
            sensor_radius: 60.0,
            metabolism: 1.0,
            color_hs: (120, 70),
        };
        let herb_idx = arena
            .spawn(Vec2::new(50.0, 50.0), Vec2::ZERO, herb_genes)
            .index as usize;

        // Spawn a Carnivore nearby
        let carn_genes = Genome {
            role: BoidRole::Carnivore,
            max_speed: 3.0,
            agility: 1.0,
            size: 1.0,
            strength: 1.0,
            sensor_radius: 60.0,
            metabolism: 1.0,
            color_hs: (0, 80),
        };
        let _carn_idx = arena
            .spawn(Vec2::new(60.0, 50.0), Vec2::ZERO, carn_genes)
            .index as usize;

        grid.build(&arena);
        update_states(&mut arena, &grid);

        assert_eq!(
            arena.states[herb_idx],
            BoidState::Flee,
            "Herbivore should flee from nearby Carnivore"
        );
    }

    #[test]
    fn test_state_transition_hunt() {
        let mut arena: BoidArena<100> = BoidArena::new();
        let mut grid: SpatialGrid<16> = SpatialGrid::new(200.0, 200.0, 60.0);

        // Spawn a Carnivore
        let carn_genes = Genome {
            role: BoidRole::Carnivore,
            max_speed: 3.0,
            agility: 1.0,
            size: 1.0,
            strength: 1.0,
            sensor_radius: 60.0,
            metabolism: 1.0,
            color_hs: (0, 80),
        };
        let carn_idx = arena
            .spawn(Vec2::new(50.0, 50.0), Vec2::ZERO, carn_genes)
            .index as usize;

        // Spawn a Herbivore nearby
        let herb_genes = Genome {
            role: BoidRole::Herbivore,
            max_speed: 3.0,
            agility: 1.0,
            size: 1.0,
            strength: 1.0,
            sensor_radius: 60.0,
            metabolism: 1.0,
            color_hs: (120, 70),
        };
        let _herb_idx = arena
            .spawn(Vec2::new(60.0, 50.0), Vec2::ZERO, herb_genes)
            .index as usize;

        grid.build(&arena);
        update_states(&mut arena, &grid);

        assert_eq!(
            arena.states[carn_idx],
            BoidState::Hunt,
            "Carnivore should hunt nearby Herbivore"
        );
    }

    #[test]
    fn test_state_forage() {
        let mut arena: BoidArena<100> = BoidArena::new();
        let grid: SpatialGrid<16> = SpatialGrid::new(200.0, 200.0, 60.0);

        // Use a Herbivore specifically (not random) since Carnivores have different logic
        let genes = Genome {
            role: BoidRole::Herbivore,
            max_speed: 3.0,
            agility: 1.0,
            size: 1.0,
            strength: 1.0,
            sensor_radius: 60.0,
            metabolism: 1.0,
            color_hs: (120, 70),
        };
        let idx = arena.spawn(Vec2::new(50.0, 50.0), Vec2::ZERO, genes).index as usize;

        // Set energy low (below 80 threshold)
        arena.energy[idx] = 40.0;

        update_states(&mut arena, &grid);

        assert_eq!(
            arena.states[idx],
            BoidState::Forage,
            "Low energy Herbivore should forage"
        );
    }

    // ============================================================================
    // INTERACTION TESTS
    // ============================================================================

    #[test]
    fn test_predation_damage() {
        let mut arena: BoidArena<100> = BoidArena::new();

        let carn_genes = Genome {
            role: BoidRole::Carnivore,
            max_speed: 3.0,
            agility: 1.0,
            size: 1.0,
            strength: 1.5,
            sensor_radius: 60.0,
            metabolism: 1.0,
            color_hs: (0, 80),
        };
        let carn_idx = arena
            .spawn(Vec2::new(50.0, 50.0), Vec2::ZERO, carn_genes)
            .index as usize;
        arena.energy[carn_idx] = 100.0;

        let herb_genes = Genome {
            role: BoidRole::Herbivore,
            max_speed: 3.0,
            agility: 1.0,
            size: 1.0,
            strength: 1.0,
            sensor_radius: 60.0,
            metabolism: 1.0,
            color_hs: (120, 70),
        };
        let herb_idx = arena
            .spawn(Vec2::new(51.0, 50.0), Vec2::ZERO, herb_genes)
            .index as usize;
        let initial_herb_energy = 100.0;
        arena.energy[herb_idx] = initial_herb_energy;

        process_predation(&mut arena);

        assert!(
            arena.energy[herb_idx] < initial_herb_energy,
            "Herbivore should lose energy"
        );
        assert!(
            arena.energy[carn_idx] > 100.0 || arena.energy[carn_idx] == 100.0,
            "Carnivore should gain energy or stay at max"
        );
    }

    #[test]
    fn test_death_cleanup() {
        let mut arena: BoidArena<100> = BoidArena::new();
        let genes = Genome::random();
        let idx = arena.spawn(Vec2::new(50.0, 50.0), Vec2::ZERO, genes).index as usize;

        arena.kill(idx);

        assert_eq!(
            arena.states[idx],
            BoidState::Dead,
            "Dead boid should be marked as Dead"
        );
        assert!(!arena.alive[idx], "Dead boid should not be alive");
    }

    // ============================================================================
    // PERFORMANCE TESTS
    // ============================================================================

    #[test]
    fn test_arena_capacity_4096() {
        let mut arena: BoidArena<4096> = BoidArena::new();
        let mut grid: SpatialGrid<32> = SpatialGrid::new(1000.0, 1000.0, 60.0);

        // Fill arena
        for i in 0..4096 {
            let x = (i % 100) as f32 * 10.0;
            let y = (i / 100) as f32 * 10.0;
            arena.spawn(Vec2::new(x, y), Vec2::ZERO, Genome::random());
        }

        assert_eq!(arena.alive_count, 4096);

        grid.build(&arena);

        // Run simulation step
        let config = SimConfig::default();
        let (births, deaths) = simulation_step(&mut arena, &grid, &config, 1000.0, 1000.0, 1.0);

        // Should complete without panic
        let _ = (births, deaths);
    }
}
