//! Comprehensive Test Suite for Antimony-Core
//!
//! This module provides extensive test coverage for:
//! - Numerical stability (NaN, Infinity, division by zero)
//! - Boundary conditions (empty arenas, full capacity)
//! - State machine transitions
//! - Spatial indexing correctness
//! - Physics calculations
//! - Ecosystem dynamics
//!
//! ## Traceability Matrix
//! | Test | Module | Function | Risk |
//! |------|--------|----------|------|
//! | test_empty_arena_operations | lib.rs | BoidArena | High - Empty state edge case |
//! | test_full_capacity_arena | lib.rs | BoidArena | High - Overflow protection |
//! | test_position_nan_handling | lib.rs | simulation_step | Critical - Runtime panic |
//! | test_velocity_bounds | lib.rs | simulation_step | High - Physics stability |
//! | test_spatial_grid_edge_cases | lib.rs | SpatialGrid | Medium - Query correctness |
//! | test_flocking_force_magnitude | lib.rs | compute_flocking_forces | High - Behavior |
//! | test_food_source_depletion | lib.rs | feed_from_sources | Medium - Game logic |
//! | test_predator_zone_damage | lib.rs | apply_predator_zones | Medium - Game logic |
//! | test_diversity_calculation | lib.rs | compute_diversity | Low - Statistics |
//! | test_season_cycle_bounds | lib.rs | SeasonCycle | Low - Time math |

use dna::{
    apply_predator_zones, compute_diversity, compute_flocking_forces, feed_from_sources,
    get_boid_color, simulation_step, trigger_earthquake, trigger_mass_extinction,
    trigger_migration, update_states, BoidArena, BoidRole, BoidState, FoodSource, Genome,
    PredatorZone, SeasonCycle, SimConfig, SpatialGrid,
};
use glam::Vec2;

// =============================================================================
// ARENA EDGE CASES
// =============================================================================

#[test]
fn test_empty_arena_operations() {
    let arena: BoidArena<100> = BoidArena::new();

    // Empty arena should handle all operations gracefully
    assert_eq!(arena.alive_count, 0);
    assert_eq!(arena.remaining_capacity(), 100);

    // Iteration should be empty
    assert_eq!(arena.iter_alive().count(), 0);

    // Diversity should return safe default
    assert_eq!(compute_diversity(&arena), 1.0);
}

#[test]
fn test_full_capacity_arena() {
    let mut arena: BoidArena<10> = BoidArena::new();

    // Fill to capacity
    for i in 0..10 {
        let handle = arena.spawn(
            Vec2::new(i as f32 * 10.0, 0.0),
            Vec2::ZERO,
            Genome::default(),
        );
        assert!(handle.is_valid(), "Should successfully spawn boid {}", i);
    }

    assert_eq!(arena.alive_count, 10);
    assert_eq!(arena.remaining_capacity(), 0);

    // Spawning beyond capacity should fail gracefully
    let overflow_handle = arena.spawn(Vec2::ZERO, Vec2::ZERO, Genome::default());
    assert!(
        !overflow_handle.is_valid(),
        "Overflow spawn should return invalid handle"
    );
    assert_eq!(
        arena.alive_count, 10,
        "Count should not increase on failed spawn"
    );
}

#[test]
fn test_spawn_recycle_slots() {
    let mut arena: BoidArena<5> = BoidArena::new();

    // Fill and kill
    let handles: Vec<_> = (0..5)
        .map(|i| arena.spawn(Vec2::new(i as f32, 0.0), Vec2::ZERO, Genome::default()))
        .collect();

    // Kill first two
    arena.kill(handles[0].index());
    arena.kill(handles[1].index());
    assert_eq!(arena.alive_count, 3);

    // New spawns should reuse slots
    let new_handle = arena.spawn(Vec2::new(100.0, 0.0), Vec2::ZERO, Genome::default());
    assert!(new_handle.is_valid());
    assert_eq!(arena.alive_count, 4);
}

// =============================================================================
// NUMERICAL STABILITY
// =============================================================================

#[test]
fn test_position_nan_handling() {
    let mut arena: BoidArena<100> = BoidArena::new();
    let mut grid: SpatialGrid<32> = SpatialGrid::new(1000.0, 1000.0, 50.0);
    let config = SimConfig::default();

    // Spawn boids with valid positions
    for _ in 0..50 {
        arena.spawn(
            Vec2::new(500.0, 500.0),
            Vec2::new(1.0, 0.0),
            Genome::random(),
        );
    }

    grid.build(&arena);

    // Run simulation for many frames
    for frame in 0..500 {
        grid.build(&arena);
        compute_flocking_forces(&mut arena, &grid, 60.0, &[]);
        let _ = simulation_step(&mut arena, &grid, &config, 1000.0, 1000.0, 0.016);

        // Check all positions are valid
        for idx in arena.iter_alive() {
            let pos = arena.positions[idx];
            assert!(!pos.x.is_nan(), "Position X became NaN at frame {}", frame);
            assert!(!pos.y.is_nan(), "Position Y became NaN at frame {}", frame);
            assert!(
                pos.x.is_finite(),
                "Position X became infinite at frame {}",
                frame
            );
            assert!(
                pos.y.is_finite(),
                "Position Y became infinite at frame {}",
                frame
            );
        }
    }
}

#[test]
fn test_velocity_bounds() {
    let mut arena: BoidArena<100> = BoidArena::new();
    let mut grid: SpatialGrid<32> = SpatialGrid::new(1000.0, 1000.0, 50.0);
    let config = SimConfig::default();

    // Create boids with extreme initial velocities
    for _ in 0..20 {
        arena.spawn(
            Vec2::new(500.0, 500.0),
            Vec2::new(1000.0, 1000.0), // Extreme velocity
            Genome::random(),
        );
    }

    // Run simulation
    for _ in 0..100 {
        grid.build(&arena);
        compute_flocking_forces(&mut arena, &grid, 60.0, &[]);
        let _ = simulation_step(&mut arena, &grid, &config, 1000.0, 1000.0, 0.016);
    }

    // Velocities should be bounded by max_speed
    for idx in arena.iter_alive() {
        let vel = arena.velocities[idx];
        let speed = vel.length();
        let max_speed = arena.genes[idx].max_speed;
        assert!(
            speed <= max_speed + 0.001, // Small epsilon for floating point
            "Velocity {} exceeds max_speed {} at index {}",
            speed,
            max_speed,
            idx
        );
    }
}

#[test]
fn test_zero_division_safety() {
    let mut arena: BoidArena<100> = BoidArena::new();
    let mut grid: SpatialGrid<32> = SpatialGrid::new(1000.0, 1000.0, 50.0);

    // Spawn two boids at exact same position (zero distance)
    arena.spawn(Vec2::new(100.0, 100.0), Vec2::ZERO, Genome::default());
    arena.spawn(Vec2::new(100.0, 100.0), Vec2::ZERO, Genome::default());

    grid.build(&arena);

    // This should not panic or produce NaN
    compute_flocking_forces(&mut arena, &grid, 60.0, &[]);

    // Verify no NaN
    for idx in arena.iter_alive() {
        assert!(!arena.scratch_accel[idx].x.is_nan());
        assert!(!arena.scratch_accel[idx].y.is_nan());
    }
}

// =============================================================================
// SPATIAL INDEXING
// =============================================================================

#[test]
fn test_spatial_grid_boundary_boids() {
    let mut arena: BoidArena<100> = BoidArena::new();
    let mut grid: SpatialGrid<32> = SpatialGrid::new(100.0, 100.0, 50.0);

    // Place boids at exact boundaries
    arena.spawn(Vec2::new(0.0, 0.0), Vec2::ZERO, Genome::default());
    arena.spawn(Vec2::new(100.0, 100.0), Vec2::ZERO, Genome::default());
    arena.spawn(Vec2::new(0.0, 100.0), Vec2::ZERO, Genome::default());
    arena.spawn(Vec2::new(100.0, 0.0), Vec2::ZERO, Genome::default());

    // Build should not panic
    grid.build(&arena);

    // Query should work
    let count = grid.count_neighbors(Vec2::new(50.0, 50.0), 100.0, &arena, usize::MAX);
    assert_eq!(count, 4, "Should find all 4 boundary boids");
}

#[test]
fn test_spatial_grid_large_radius_query() {
    let mut arena: BoidArena<100> = BoidArena::new();
    let mut grid: SpatialGrid<32> = SpatialGrid::new(1000.0, 1000.0, 50.0);

    // Scatter boids
    for i in 0..50 {
        arena.spawn(
            Vec2::new((i % 10) as f32 * 100.0, (i / 10) as f32 * 100.0),
            Vec2::ZERO,
            Genome::default(),
        );
    }

    grid.build(&arena);

    // Query with huge radius should find all
    let count = grid.count_neighbors(Vec2::new(500.0, 500.0), 2000.0, &arena, usize::MAX);
    assert_eq!(count, 50, "Large radius query should find all boids");
}

#[test]
fn test_spatial_grid_zero_radius_query() {
    let mut arena: BoidArena<100> = BoidArena::new();
    let mut grid: SpatialGrid<32> = SpatialGrid::new(1000.0, 1000.0, 50.0);

    arena.spawn(Vec2::new(100.0, 100.0), Vec2::ZERO, Genome::default());
    grid.build(&arena);

    // Zero radius should find nothing (or only exact matches)
    let count = grid.count_neighbors(Vec2::new(100.0, 100.0), 0.0, &arena, 0);
    assert!(count == 0, "Zero radius query should find no neighbors");
}

// =============================================================================
// STATE MACHINE
// =============================================================================

#[test]
fn test_all_state_transitions() {
    let mut arena: BoidArena<100> = BoidArena::new();
    let mut grid: SpatialGrid<32> = SpatialGrid::new(1000.0, 1000.0, 50.0);

    // Test WANDER (default high energy, no threats)
    let wander_genes = Genome {
        role: BoidRole::Herbivore,
        ..Genome::default()
    };
    let wander_idx = arena
        .spawn(Vec2::new(100.0, 100.0), Vec2::ZERO, wander_genes)
        .index();
    arena.energy[wander_idx] = 100.0;

    // Test FORAGE (low energy)
    let forage_genes = Genome {
        role: BoidRole::Herbivore,
        ..Genome::default()
    };
    let forage_idx = arena
        .spawn(Vec2::new(300.0, 100.0), Vec2::ZERO, forage_genes)
        .index();
    arena.energy[forage_idx] = 50.0;

    // Test REPRODUCE (high energy)
    let repro_genes = Genome {
        role: BoidRole::Herbivore,
        ..Genome::default()
    };
    let repro_idx = arena
        .spawn(Vec2::new(500.0, 100.0), Vec2::ZERO, repro_genes)
        .index();
    arena.energy[repro_idx] = 200.0;

    grid.build(&arena);
    update_states(&mut arena, &grid);

    assert_eq!(arena.states[wander_idx], BoidState::Wander);
    assert_eq!(arena.states[forage_idx], BoidState::Forage);
    assert_eq!(arena.states[repro_idx], BoidState::Reproduce);
}

#[test]
fn test_flee_priority_over_other_states() {
    let mut arena: BoidArena<100> = BoidArena::new();
    let mut grid: SpatialGrid<32> = SpatialGrid::new(1000.0, 1000.0, 60.0);

    // High energy herbivore (would reproduce if no threat)
    let herb_genes = Genome {
        role: BoidRole::Herbivore,
        sensor_radius: 100.0,
        ..Genome::default()
    };
    let herb_idx = arena
        .spawn(Vec2::new(100.0, 100.0), Vec2::ZERO, herb_genes)
        .index();
    arena.energy[herb_idx] = 200.0; // High energy

    // Nearby carnivore
    let carn_genes = Genome {
        role: BoidRole::Carnivore,
        ..Genome::default()
    };
    arena.spawn(Vec2::new(110.0, 100.0), Vec2::ZERO, carn_genes);

    grid.build(&arena);
    update_states(&mut arena, &grid);

    // Should FLEE despite high energy
    assert_eq!(
        arena.states[herb_idx],
        BoidState::Flee,
        "Flee should override Reproduce"
    );
}

// =============================================================================
// FOOD AND ENERGY
// =============================================================================

#[test]
fn test_food_source_depletion() {
    let mut arena: BoidArena<100> = BoidArena::new();
    let mut sources = vec![FoodSource {
        position: Vec2::new(100.0, 100.0),
        radius: 50.0,
        energy: 10.0,
        max_energy: 100.0,
        regen_rate: 0.0, // No regen for test
        depleted_timer: 0.0,
    }];

    // Spawn hungry herbivore at food source
    let genes = Genome {
        role: BoidRole::Herbivore,
        ..Genome::default()
    };
    let idx = arena
        .spawn(Vec2::new(100.0, 100.0), Vec2::ZERO, genes)
        .index();
    arena.energy[idx] = 50.0;

    let initial_food = sources[0].energy;
    let initial_energy = arena.energy[idx];

    let season = SeasonCycle::new();
    feed_from_sources(&mut arena, &mut sources, &season);

    assert!(sources[0].energy < initial_food, "Food should deplete");
    assert!(
        arena.energy[idx] > initial_energy,
        "Boid should gain energy"
    );
}

#[test]
fn test_food_source_regeneration() {
    let mut sources = vec![FoodSource {
        position: Vec2::new(100.0, 100.0),
        radius: 50.0,
        energy: 50.0,
        max_energy: 100.0,
        regen_rate: 10.0, // High regen for test
        depleted_timer: 0.0,
    }];

    let mut arena: BoidArena<100> = BoidArena::new(); // Empty - no eating
    let season = SeasonCycle::new();

    feed_from_sources(&mut arena, &mut sources, &season);

    assert!(sources[0].energy > 50.0, "Food should regenerate");
    assert!(sources[0].energy <= 100.0, "Food should not exceed max");
}

// =============================================================================
// PREDATOR ZONES
// =============================================================================

#[test]
fn test_predator_zone_damage_scaling() {
    let mut arena: BoidArena<100> = BoidArena::new();
    let predators = vec![PredatorZone {
        position: Vec2::new(100.0, 100.0),
        radius: 50.0,
        intensity: 10.0,
        active: true,
        lifetime: 100.0,
    }];

    // Boid at center (max damage)
    let idx_center = arena
        .spawn(Vec2::new(100.0, 100.0), Vec2::ZERO, Genome::default())
        .index();
    arena.energy[idx_center] = 100.0;

    // Boid at edge (min damage)
    let idx_edge = arena
        .spawn(Vec2::new(149.0, 100.0), Vec2::ZERO, Genome::default())
        .index();
    arena.energy[idx_edge] = 100.0;

    // Boid outside (no damage)
    let idx_outside = arena
        .spawn(Vec2::new(200.0, 100.0), Vec2::ZERO, Genome::default())
        .index();
    arena.energy[idx_outside] = 100.0;

    apply_predator_zones(&mut arena, &predators);

    let dmg_center = 100.0 - arena.energy[idx_center];
    let dmg_edge = 100.0 - arena.energy[idx_edge];
    let dmg_outside = 100.0 - arena.energy[idx_outside];

    assert!(
        dmg_center > dmg_edge,
        "Center should take more damage than edge"
    );
    assert!(dmg_outside == 0.0, "Outside should take no damage");
}

#[test]
fn test_inactive_predator_zone() {
    let mut arena: BoidArena<100> = BoidArena::new();
    let predators = vec![PredatorZone {
        position: Vec2::new(100.0, 100.0),
        radius: 50.0,
        intensity: 100.0,
        active: false, // Inactive
        lifetime: 0.0,
    }];

    let idx = arena
        .spawn(Vec2::new(100.0, 100.0), Vec2::ZERO, Genome::default())
        .index();
    arena.energy[idx] = 100.0;

    apply_predator_zones(&mut arena, &predators);

    assert_eq!(
        arena.energy[idx], 100.0,
        "Inactive predator should deal no damage"
    );
}

// =============================================================================
// DIVERSITY AND ECOSYSTEM
// =============================================================================

#[test]
fn test_diversity_monoculture() {
    let mut arena: BoidArena<100> = BoidArena::new();

    // All herbivores with identical genes
    for i in 0..50 {
        let mut genes = Genome::default();
        genes.role = BoidRole::Herbivore;
        genes.max_speed = 3.0; // Same speed
        arena.spawn(Vec2::new(i as f32, 0.0), Vec2::ZERO, genes);
    }

    let diversity = compute_diversity(&arena);
    assert!(
        diversity < 0.5,
        "Monoculture should have low diversity: {}",
        diversity
    );
}

#[test]
fn test_diversity_balanced_ecosystem() {
    let mut arena: BoidArena<100> = BoidArena::new();

    // Equal distribution of roles with varied speeds
    for i in 0..20 {
        let mut genes = Genome::default();
        genes.role = BoidRole::Herbivore;
        genes.max_speed = 2.0 + (i as f32 * 0.1);
        arena.spawn(Vec2::new(i as f32, 0.0), Vec2::ZERO, genes);
    }
    for i in 0..20 {
        let mut genes = Genome::default();
        genes.role = BoidRole::Carnivore;
        genes.max_speed = 2.0 + (i as f32 * 0.1);
        arena.spawn(Vec2::new(i as f32, 10.0), Vec2::ZERO, genes);
    }
    for i in 0..20 {
        let mut genes = Genome::default();
        genes.role = BoidRole::Scavenger;
        genes.max_speed = 2.0 + (i as f32 * 0.1);
        arena.spawn(Vec2::new(i as f32, 20.0), Vec2::ZERO, genes);
    }

    let diversity = compute_diversity(&arena);
    assert!(
        diversity > 0.6,
        "Balanced ecosystem should have high diversity: {}",
        diversity
    );
}

// =============================================================================
// SEASON CYCLE
// =============================================================================

#[test]
fn test_season_cycle_bounds() {
    let mut season = SeasonCycle::new();

    // Test phase stays in 0-1
    for _ in 0..10000 {
        season.update(1.0);
        let phase = season.phase();
        assert!(phase >= 0.0 && phase < 1.0, "Phase {} out of bounds", phase);
    }
}

#[test]
fn test_season_food_multiplier_range() {
    let mut season = SeasonCycle::new();

    let mut min_mult = f32::MAX;
    let mut max_mult = f32::MIN;

    for _ in 0..season.period as usize {
        season.update(1.0);
        let mult = season.food_multiplier();
        min_mult = min_mult.min(mult);
        max_mult = max_mult.max(mult);
    }

    assert!(
        min_mult >= 0.3,
        "Min multiplier should be >= 0.3: {}",
        min_mult
    );
    assert!(
        max_mult <= 2.0,
        "Max multiplier should be <= 2.0: {}",
        max_mult
    );
}

// =============================================================================
// MASS EXTINCTION
// =============================================================================

#[test]
fn test_mass_extinction_survival() {
    let mut arena: BoidArena<1000> = BoidArena::new();

    // Populate
    for _ in 0..500 {
        arena.spawn(Vec2::new(500.0, 500.0), Vec2::ZERO, Genome::random());
    }

    assert_eq!(arena.alive_count, 500);

    // 90% extinction
    trigger_mass_extinction(&mut arena, 0.9, 1000.0, 1000.0);

    // Should have survivors + founders
    assert!(
        arena.alive_count > 10,
        "Some should survive: {}",
        arena.alive_count
    );
    assert!(
        arena.alive_count < 100,
        "Most should die: {}",
        arena.alive_count
    );
}

// =============================================================================
// EVENT TRIGGERS
// =============================================================================

#[test]
fn test_migration_applies_velocity() {
    let mut arena: BoidArena<100> = BoidArena::new();

    for _ in 0..10 {
        arena.spawn(Vec2::new(500.0, 500.0), Vec2::ZERO, Genome::default());
    }

    trigger_migration(&mut arena, Vec2::new(1.0, 0.0), 5.0);

    for idx in arena.iter_alive() {
        assert!(
            arena.velocities[idx].x > 4.0,
            "Migration should add velocity"
        );
    }
}

#[test]
fn test_earthquake_randomizes_velocity() {
    let mut arena: BoidArena<100> = BoidArena::new();

    for _ in 0..10 {
        arena.spawn(Vec2::new(500.0, 500.0), Vec2::ZERO, Genome::default());
    }

    trigger_earthquake(&mut arena);

    let mut has_different = false;
    let first_vel = arena.velocities[0];
    for idx in arena.iter_alive() {
        if arena.velocities[idx] != first_vel {
            has_different = true;
        }
    }
    assert!(
        has_different,
        "Earthquake should randomize velocities differently"
    );
}

// =============================================================================
// COLOR / RENDERING
// =============================================================================

#[test]
fn test_boid_color_bounds() {
    let mut arena: BoidArena<100> = BoidArena::new();

    // Test with various energy levels
    for energy in [0.0, 50.0, 100.0, 200.0, 500.0] {
        let idx = arena
            .spawn(Vec2::new(0.0, 0.0), Vec2::ZERO, Genome::default())
            .index();
        arena.energy[idx] = energy;

        let (hue, sat, light) = get_boid_color(&arena, idx);

        assert!(hue <= 360, "Hue {} out of range", hue);
        assert!(sat <= 100, "Saturation {} out of range", sat);
        assert!(
            light >= 25 && light <= 80,
            "Lightness {} out of range",
            light
        );
    }
}
