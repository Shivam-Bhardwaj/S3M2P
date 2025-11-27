use glam::Vec2;
use rand::Rng;

pub mod heliosphere;
pub use heliosphere::*;

// ============================================================================
// CORE TYPES
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct Obstacle {
    pub center: Vec2,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Genome {
    pub max_speed: f32,
    pub metabolism: f32, // 0.8-1.2: lower = more efficient
}

impl Genome {
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            max_speed: rng.gen_range(2.0..=4.0),
            metabolism: rng.gen_range(0.8..=1.2),
        }
    }

    pub fn mutate(&self) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            max_speed: (self.max_speed * rng.gen_range(0.95..=1.05)).clamp(1.5, 5.0),
            metabolism: (self.metabolism * rng.gen_range(0.95..=1.05)).clamp(0.7, 1.3),
        }
    }

    /// Hue from speed (blue=slow, red=fast), saturation from metabolism
    #[inline]
    pub fn color_hs(&self) -> (u16, u8) {
        let speed_norm = ((self.max_speed - 1.5) / 3.5).clamp(0.0, 1.0);
        let hue = ((1.0 - speed_norm) * 240.0) as u16;
        let sat = (50.0 + (self.metabolism - 0.7) * 83.0) as u8;
        (hue, sat.clamp(50, 100))
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
    pub const INVALID: Self = Self { index: u16::MAX, generation: 0 };
    
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.index != u16::MAX
    }
}

/// Structure of Arrays layout for cache-friendly iteration
/// All arrays are the same length (CAPACITY)
pub struct BoidArena<const CAPACITY: usize> {
    // SoA layout - each field in its own array for cache locality
    pub positions: [Vec2; CAPACITY],
    pub velocities: [Vec2; CAPACITY],
    pub genes: [Genome; CAPACITY],
    pub energy: [f32; CAPACITY],
    pub age: [f32; CAPACITY],
    pub generation: [u16; CAPACITY],
    
    // Metadata
    pub alive: [bool; CAPACITY],
    gen: [u16; CAPACITY],  // Generation counter for handles
    
    // Free list (indices of dead slots)
    free_list: [u16; CAPACITY],
    free_count: usize,
    
    // Active count for fast iteration
    pub alive_count: usize,
    
    // Pre-allocated scratch buffers (avoid per-frame allocations)
    pub scratch_accel: [Vec2; CAPACITY],
    pub scratch_density: [u8; CAPACITY],
}

impl<const CAPACITY: usize> BoidArena<CAPACITY> {
    pub fn new() -> Self {
        let mut arena = Self {
            positions: [Vec2::ZERO; CAPACITY],
            velocities: [Vec2::ZERO; CAPACITY],
            genes: [Genome::default(); CAPACITY],
            energy: [0.0; CAPACITY],
            age: [0.0; CAPACITY],
            generation: [0; CAPACITY],
            alive: [false; CAPACITY],
            gen: [0; CAPACITY],
            free_list: [0; CAPACITY],
            free_count: CAPACITY,
            alive_count: 0,
            scratch_accel: [Vec2::ZERO; CAPACITY],
            scratch_density: [0; CAPACITY],
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
    #[inline]
    pub fn kill(&mut self, idx: usize) {
        if idx < CAPACITY && self.alive[idx] {
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
        let neighbor_count = grid.query_neighbors(pos, vision_radius, arena, idx, &mut neighbors);
        
        // Store density for population dynamics
        arena.scratch_density[idx] = neighbor_count.min(255) as u8;
        
        if neighbor_count == 0 {
            // Only obstacle avoidance
            arena.scratch_accel[idx] = compute_obstacle_avoidance(pos, obstacles);
            continue;
        }
        
        let mut cohesion = Vec2::ZERO;
        let mut alignment = Vec2::ZERO;
        let mut separation = Vec2::ZERO;
        
        for i in 0..neighbor_count {
            let other_idx = neighbors[i] as usize;
            let other_pos = arena.positions[other_idx];
            let other_vel = arena.velocities[other_idx];
            
            let diff = other_pos - pos;
            let dist = diff.length();
            
            cohesion += other_pos;
            alignment += other_vel;
            if dist > 0.001 {
                separation -= diff / dist;
            }
        }
        
        let n = neighbor_count as f32;
        cohesion = cohesion / n - pos;
        alignment /= n;
        separation /= n;
        
        let flocking = cohesion * 1.0 + alignment * 1.0 + separation * 1.5;
        let avoidance = compute_obstacle_avoidance(pos, obstacles);
        
        arena.scratch_accel[idx] = flocking + avoidance;
    }
}

#[inline]
fn compute_obstacle_avoidance(pos: Vec2, obstacles: &[Obstacle]) -> Vec2 {
    let mut force = Vec2::ZERO;
    const BUFFER: f32 = 50.0;
    
    for obs in obstacles {
        let d = pos.distance(obs.center);
        if d < obs.radius + BUFFER && d > 0.001 {
            let repulsion = (pos - obs.center).normalize();
            force += repulsion * (100.0 / d);
        }
    }
    force
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
            reproduction_threshold: 120.0,  // Easier to reproduce
            reproduction_cost: 40.0,        // Cheaper reproduction
            max_age: 2000.0,                // Longer lifespan
            base_mortality: 0.00002,        // Much gentler base mortality
            starvation_threshold: 10.0,     // Only die when very low energy
        }
    }
}

/// Single simulation step - zero heap allocations
pub fn simulation_step<const CAP: usize, const CELL_CAP: usize>(
    arena: &mut BoidArena<CAP>,
    _grid: &SpatialGrid<CELL_CAP>,
    config: &SimConfig,
    width: f32,
    height: f32,
    dt: f32,
) -> (usize, usize) { // returns (births, deaths)
    let mut rng = rand::thread_rng();
    let mut births = 0usize;
    let mut deaths = 0usize;
    let population = arena.alive_count;
    
    // Collect reproduction candidates first (to avoid borrowing issues)
    let mut reproduce_indices = [0u16; 128];
    let mut reproduce_count = 0;
    
    // Phase 1: Apply forces and update state
    for idx in 0..CAP {
        if !arena.alive[idx] {
            continue;
        }
        
        // Apply acceleration
        let accel = arena.scratch_accel[idx] * 0.05;
        arena.velocities[idx] += accel;
        
        // Limit speed
        let max_speed = arena.genes[idx].max_speed;
        let speed = arena.velocities[idx].length();
        if speed > max_speed {
            arena.velocities[idx] = arena.velocities[idx] / speed * max_speed;
        }
        
        // Update position
        arena.positions[idx] += arena.velocities[idx] * dt;
        
        // Wrap around
        if arena.positions[idx].x < 0.0 { arena.positions[idx].x += width; }
        if arena.positions[idx].x >= width { arena.positions[idx].x -= width; }
        if arena.positions[idx].y < 0.0 { arena.positions[idx].y += height; }
        if arena.positions[idx].y >= height { arena.positions[idx].y -= height; }
        
        // Metabolism (very low drain - survival is easier)
        let metabolism_cost = speed * 0.002 * arena.genes[idx].metabolism;
        arena.energy[idx] -= metabolism_cost;
        
        // Aging
        arena.age[idx] += dt;
        
        // Check reproduction
        if arena.energy[idx] > config.reproduction_threshold 
            && reproduce_count < 128
            && population + reproduce_count < config.carrying_capacity
        {
            reproduce_indices[reproduce_count] = idx as u16;
            reproduce_count += 1;
        }
    }
    
    // Phase 2: Reproduction (separate pass to avoid borrow conflicts)
    for i in 0..reproduce_count {
        let parent_idx = reproduce_indices[i] as usize;
        if arena.alive[parent_idx] && arena.energy[parent_idx] > config.reproduction_threshold {
            let handle = arena.spawn_child(parent_idx);
            if handle.is_valid() {
                births += 1;
            }
        }
    }
    
    // Phase 3: Death checks
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
    Famine,           // Food stops regenerating
    Bloom,            // Food regenerates 3x faster
    PredatorSpawn,    // New predator zone appears
    Migration,        // Boids get pushed in a direction
    Earthquake,       // Randomize all velocities
}

/// Seasonal cycle affects food and mortality
#[derive(Clone, Copy, Debug)]
pub struct SeasonCycle {
    pub time: f32,
    pub period: f32, // ~30 seconds per season
}

impl SeasonCycle {
    pub fn new() -> Self {
        Self { time: 0.0, period: 1800.0 } // 30 second seasons
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
        if phase < 0.25 { "SPRING" }
        else if phase < 0.5 { "SUMMER" }
        else if phase < 0.75 { "AUTUMN" }
        else { "WINTER" }
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
            arena.velocities[idx] = Vec2::new(
                rng.gen_range(-3.0..3.0),
                rng.gen_range(-3.0..3.0),
            );
            // Stress from earthquake
            arena.energy[idx] -= 5.0;
        }
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
        arena.spawn(Vec2::new(10.0, 10.0), Vec2::ZERO, Genome::random());
        arena.spawn(Vec2::new(15.0, 10.0), Vec2::ZERO, Genome::random());
        arena.spawn(Vec2::new(100.0, 100.0), Vec2::ZERO, Genome::random());
        
        let mut grid: SpatialGrid<16> = SpatialGrid::new(200.0, 200.0, 50.0);
        grid.build(&arena);
        
        let count = grid.count_neighbors(Vec2::new(10.0, 10.0), 20.0, &arena, 0);
        assert_eq!(count, 1); // Should find boid at (15, 10) but not self
    }
}
