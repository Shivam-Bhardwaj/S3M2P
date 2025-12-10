//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: lib.rs | SIMULATION/CORE/SIMULATION_ENGINE/src/lib.rs
//! PURPOSE: Simulation runtime engine for boid/particle systems
//! MODIFIED: 2025-12-09
//! LAYER: CORE → SIMULATION_ENGINE
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! SIMULATION_ENGINE orchestrates particle and agent simulations:
//! - BoidArena management (spawn, kill, iterate)
//! - SpatialGrid queries (neighbor finding)
//! - State machine updates (Hunt, Flee, Forage, etc.)
//! - Force computation (flocking, avoidance, attraction)
//! - Time stepping and physics integration
//!
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │ ARCHITECTURE                                                                │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │   SimulationEngine                                                          │
//! │       │                                                                     │
//! │       ├── BoidArena<CAPACITY>  (DNA/data/arena)                             │
//! │       ├── SpatialGrid<CELL_CAP> (DNA/data/spatial_grid)                     │
//! │       ├── SimConfig            (DNA/lib.rs)                                 │
//! │       └── FoodSource[]         (DNA/lib.rs)                                 │
//! │                                                                             │
//! │   Per-frame pipeline:                                                       │
//! │   1. grid.build()              - Rebuild spatial index                      │
//! │   2. update_states()           - State machine transitions                  │
//! │   3. compute_flocking_forces() - Calculate accelerations                    │
//! │   4. simulation_step()         - Physics, reproduction, death               │
//! │   5. feed_from_sources()       - Energy transfer from food                  │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//!
//! DEPENDS ON:
//!   • DNA/data/arena          → BoidArena, BoidHandle
//!   • DNA/data/spatial_grid   → SpatialGrid
//!   • DNA/lib.rs              → Genome, BoidState, SimConfig, FoodSource
//!
//! USED BY:
//!   • WELCOME (too.foo)       → Bubble simulation
//!   • SIMULATIONS/CHLADNI     → Particle visualization
//!
//! ═══════════════════════════════════════════════════════════════════════════════

// ─────────────────────────────────────────────────────────────────────────────────
// CODE BELOW - Optimized for ML development
// ─────────────────────────────────────────────────────────────────────────────────

// Re-export DNA types for convenience
pub use dna::{
    BoidArena, BoidHandle, BoidRole, BoidState, FoodSource, Genome, Obstacle, PredatorZone,
    SeasonCycle, SimConfig, SpatialGrid,
};

// Re-export simulation functions
pub use dna::{
    apply_predator_zones, compute_diversity, compute_flocking_forces, feed_from_sources,
    get_boid_color, process_predation, process_scavenging, simulation_step, trigger_earthquake,
    trigger_mass_extinction, trigger_migration, update_states,
};

/// Default arena capacity
pub const DEFAULT_CAPACITY: usize = 2048;

/// Default spatial grid cell capacity
pub const DEFAULT_CELL_CAPACITY: usize = 32;

/// Create a new simulation with default parameters
pub fn create_simulation<const CAPACITY: usize, const CELL_CAPACITY: usize>(
    width: f32,
    height: f32,
    cell_size: f32,
) -> (BoidArena<CAPACITY>, SpatialGrid<CELL_CAPACITY>, SimConfig) {
    let arena = BoidArena::new();
    let grid = SpatialGrid::new(width, height, cell_size);
    let config = SimConfig::default();
    (arena, grid, config)
}

/// Seed initial population randomly
pub fn seed_population<const CAPACITY: usize>(
    arena: &mut BoidArena<CAPACITY>,
    count: usize,
    width: f32,
    height: f32,
) {
    use dna::{random_in_rect, random_velocity};

    for _ in 0..count {
        let pos = random_in_rect(width, height);
        let vel = random_velocity(2.0);
        let genes = Genome::random();
        arena.spawn(pos, vel, genes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_simulation() {
        let (arena, grid, config): (BoidArena<1024>, SpatialGrid<16>, SimConfig) =
            create_simulation(800.0, 600.0, 50.0);

        assert_eq!(arena.alive_count, 0);
        assert_eq!(config.carrying_capacity, 800);
        let _ = grid; // Just verify it was created
    }

    #[test]
    fn test_seed_population() {
        let mut arena: BoidArena<1024> = BoidArena::new();
        seed_population(&mut arena, 100, 800.0, 600.0);

        assert_eq!(arena.alive_count, 100);
    }
}
