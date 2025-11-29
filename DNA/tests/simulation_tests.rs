use dna::{
    compute_diversity, compute_flocking_forces, process_predation, process_scavenging,
    simulation_step, trigger_mass_extinction, BoidArena, BoidRole, Genome, SimConfig, SpatialGrid,
};
use glam::Vec2;

// Helper to populate arena with specific roles
fn populate_arena_specific<const CAP: usize>(
    arena: &mut BoidArena<CAP>,
    count: usize,
    role: BoidRole,
) {
    for i in 0..count {
        let pos = Vec2::new(i as f32 * 2.0, 0.0);
        let mut genes = Genome::default();
        genes.role = role;
        arena.spawn(pos, Vec2::ZERO, genes);
    }
}

#[test]
fn test_mass_extinction_mechanics() {
    let mut arena: BoidArena<1000> = BoidArena::new();
    let width = 1000.0;
    let height = 1000.0;

    // 1. Populate
    for _ in 0..500 {
        arena.spawn(Vec2::new(500.0, 500.0), Vec2::ZERO, Genome::random());
    }

    assert_eq!(arena.alive_count, 500);

    // 2. Trigger 90% extinction
    trigger_mass_extinction(&mut arena, 0.9, width, height);

    // 3. Check results
    // Should be roughly 50 survivors (10%) + 10 founders = ~60
    // Since it's probabilistic, we use a range
    assert!(
        arena.alive_count < 150,
        "Too many survivors: {}",
        arena.alive_count
    );
    assert!(
        arena.alive_count > 10,
        "Too few survivors (founders failed?): {}",
        arena.alive_count
    );

    // 4. Verify founders were seeded
    // Founders are seeded with random velocities, so they shouldn't be all zero if we had them
    let moving_count = arena
        .iter_alive()
        .filter(|&i| arena.velocities[i].length_squared() > 0.0)
        .count();
    assert!(moving_count > 0, "No moving boids found after extinction");
}

#[test]
fn test_diversity_metric() {
    let mut arena: BoidArena<100> = BoidArena::new();

    // Case 1: Monoculture (Low Diversity)
    populate_arena_specific(&mut arena, 50, BoidRole::Herbivore);
    let div_low = compute_diversity(&arena);

    // Clear arena for next case (manual clear since no method exposed for bulk clear)
    for i in 0..100 {
        if arena.alive[i] {
            arena.kill(i);
        }
    }

    // Case 2: Balanced (High Diversity)
    populate_arena_specific(&mut arena, 20, BoidRole::Herbivore);
    populate_arena_specific(&mut arena, 20, BoidRole::Carnivore);
    populate_arena_specific(&mut arena, 20, BoidRole::Scavenger);

    // Manually vary speeds to increase trait diversity
    // Fix borrow checker: Collect indices first
    let indices: Vec<usize> = arena.iter_alive().collect();
    for &i in &indices {
        arena.genes[i].max_speed = (i % 5) as f32 + 2.0;
    }

    let div_high = compute_diversity(&arena);

    assert!(
        div_high > div_low,
        "Mixed population should have higher diversity than monoculture. Low: {}, High: {}",
        div_low,
        div_high
    );
    assert!(
        div_high > 0.5,
        "Balanced population should have good diversity score"
    );
}

#[test]
fn test_simulation_stability_long_run() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Run for 1000 frames and ensure no panics or NaN values
    let mut arena: BoidArena<200> = BoidArena::new();
    let mut grid: SpatialGrid<32> = SpatialGrid::new(1000.0, 1000.0, 50.0);
    let config = SimConfig::default();
    let width = 1000.0;
    let height = 1000.0;

    // Populate with random positions to avoid division by zero (stacked boids)
    for _ in 0..100 {
        arena.spawn(
            Vec2::new(rng.gen_range(0.0..width), rng.gen_range(0.0..height)),
            Vec2::new(1.0, 1.0),
            Genome::random(),
        );
    }

    for _frame in 0..1000 {
        grid.build(&arena);

        // We need to compute flocking forces manually as they are separated in the main loop
        // This mimics the main loop structure roughly for testing
        compute_flocking_forces(&mut arena, &grid, 60.0, &[]);

        let (_births, _deaths) = simulation_step(
            &mut arena, &grid, &config, width, height, 0.016, // ~60fps
        );

        // Sanity checks
        assert!(arena.alive_count <= 200, "Exceeded capacity!");

        // Check for NaN
        for idx in arena.iter_alive() {
            let pos = arena.positions[idx];
            assert!(
                !pos.x.is_nan() && !pos.y.is_nan(),
                "NaN position detected at frame {}",
                _frame
            );

            let vel = arena.velocities[idx];
            assert!(
                !vel.x.is_nan() && !vel.y.is_nan(),
                "NaN velocity detected at frame {}",
                _frame
            );
        }
    }
}

#[test]
fn test_predator_prey_dynamics() {
    let mut arena: BoidArena<100> = BoidArena::new();
    let _grid: SpatialGrid<16> = SpatialGrid::new(200.0, 200.0, 50.0);

    // 1 Carnivore vs 1 Herbivore (Trapped in small area)
    let carn_genes = Genome {
        role: BoidRole::Carnivore,
        max_speed: 4.0, // Faster
        strength: 2.0,
        ..Genome::default()
    };

    let herb_genes = Genome {
        role: BoidRole::Herbivore,
        max_speed: 2.0, // Slower
        strength: 0.5,
        ..Genome::default()
    };

    // Spawn scavenger first to avoid overwriting the corpse later
    let scav_genes = Genome {
        role: BoidRole::Scavenger,
        ..Genome::default()
    };
    let scav_idx = arena
        .spawn(Vec2::new(12.0, 10.0), Vec2::ZERO, scav_genes)
        .index();

    let carn_idx = arena
        .spawn(Vec2::new(10.0, 10.0), Vec2::ZERO, carn_genes)
        .index();
    let herb_idx = arena
        .spawn(Vec2::new(12.0, 10.0), Vec2::ZERO, herb_genes)
        .index(); // Close by

    // Manually process predation logic
    process_predation(&mut arena);

    // Carnivore should have damaged herbivore
    assert!(
        arena.energy[herb_idx] < 100.0,
        "Herbivore should take damage from adjacent carnivore"
    );

    // Kill herbivore manually to test scavenging
    arena.energy[herb_idx] = -10.0;
    arena.kill(herb_idx);

    // Remove Carnivore to prevent it from attacking Scavenger during this test phase
    // (Since we are testing scavenging, not continuous combat)
    arena.kill(carn_idx);

    // Reset Scavenger energy to baseline 100.0 in case it was damaged
    arena.energy[scav_idx] = 100.0;

    process_scavenging(&mut arena);

    // Scavenger should eat the dead body (which is the herbivore we just killed)
    // Initial energy is 100.0, gain is 15.0
    assert!(
        arena.energy[scav_idx] > 100.0,
        "Scavenger should gain energy from corpse. Current: {}",
        arena.energy[scav_idx]
    );
}
