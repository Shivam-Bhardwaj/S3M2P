# Core - Simulation Engine

The shared foundation for all simulation projects. Zero-allocation, cache-friendly design.

## Module Overview

| Module | Purpose |
|--------|---------|
| `lib.rs` | Core types: `BoidArena`, `SpatialGrid`, `Genome`, flocking |
| `heliosphere.rs` | Heliospheric boundary models |
| `heliosphere_model.rs` | Parker spiral, termination shock |
| `solar_wind.rs` | Solar wind particle simulation |
| `coordinates.rs` | Coordinate system transforms |
| `spatial.rs` | Additional spatial utilities |
| `zones.rs` | Zone/exclusion area logic |
| `interaction.rs` | Entity interaction effects |
| `statistics.rs` | Population metrics |
| `color.rs` | HSL color utilities |
| `random.rs` | RNG helpers |

## Key Types

### BoidArena<const CAPACITY: usize>
```rust
// SoA layout for cache-friendly iteration
pub positions: Vec<Vec2>,
pub velocities: Vec<Vec2>,
pub genes: Vec<Genome>,
pub states: Vec<BoidState>,
pub energy: Vec<f32>,
// ...

// O(1) operations
fn spawn(&mut self, pos, vel, genes) -> BoidHandle
fn kill(&mut self, idx: usize)
fn iter_alive(&self) -> impl Iterator<Item = usize>
```

### SpatialGrid<const CELL_CAPACITY: usize>
```rust
fn build<const CAP>(&mut self, arena: &BoidArena<CAP>)
fn query_neighbors(pos, radius, arena, exclude, output) -> count
fn count_neighbors(pos, radius, arena, exclude) -> usize
```

### Genome
```rust
pub role: BoidRole,        // Herbivore, Carnivore, Scavenger
pub max_speed: f32,        // 2.0 - 6.0
pub agility: f32,          // 0.5 - 2.0
pub size: f32,             // 0.5 - 2.0
pub strength: f32,         // 0.5 - 2.0
pub sensor_radius: f32,    // 40.0 - 120.0
pub metabolism: f32,       // 0.7 - 1.3

fn mutate(&self) -> Self   // 5 evolutionary events
```

## Simulation Pipeline

```
1. update_states()         - State machine transitions
2. compute_flocking_forces() - Per-state behavior
3. simulation_step()       - Physics, reproduction, death
4. process_predation()     - Carnivore attacks
5. process_scavenging()    - Corpse consumption
6. feed_from_sources()     - Food zone energy transfer
```

## Testing

```bash
cargo test -p core
cargo test -p core -- --nocapture  # See println output
```

Key test categories:
- `test_arena_*`: Entity management
- `test_spatial_*`: Grid queries
- `test_state_*`: State machine
- `test_predation_*`: Interactions
- `test_genome_*`: Evolution

## Performance Guidelines

1. **No allocations in hot paths**: Use pre-sized `scratch_*` buffers
2. **Stack-allocated neighbor buffers**: `let mut neighbors = [0u16; 64];`
3. **Avoid `Vec::push` in loops**: Pre-allocate or use fixed arrays
4. **Use `#[inline]` for small per-entity functions**
5. **Guard against NaN**: Check vector length before normalize

## Common Changes

### Adding a new BoidState
1. Add variant to `BoidState` enum
2. Add transition logic in `update_states()`
3. Add force calculation in `compute_flocking_forces()`
4. Update tests

### Adding a genome trait
1. Add field to `Genome` struct
2. Update `Genome::default()` and `Genome::random()`
3. Add to mutation logic in `mutate()`
4. Update `compute_color_hs()` if visual
5. Add tests for bounds and mutation

### Adding an interaction type
1. Create new function like `process_predation()`
2. Call from `simulation_step()` at appropriate phase
3. Add tests
