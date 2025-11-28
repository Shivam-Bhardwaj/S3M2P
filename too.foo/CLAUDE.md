# too.foo - Boid Ecosystem Visualization

Interactive ecosystem simulation with predator/prey dynamics, genetics, and evolution.

## Build & Run

```bash
# Development (hot reload)
trunk serve too.foo/index.html --open

# Production build
trunk build --release too.foo/index.html

# Output in too.foo/dist/
```

## Architecture

```
too.foo/
  src/
    main.rs      # WASM entry, event loop, rendering
  index.html     # Entry point
  assets/        # Static assets
```

## Core Integration

This project heavily uses `core` crate:

```rust
use core::{
    BoidArena, BoidHandle, BoidRole, BoidState, Genome,
    SpatialGrid, SimConfig,
    compute_flocking_forces, simulation_step, update_states,
    FoodSource, SeasonCycle,
    feed_from_sources, compute_diversity,
};
```

## Simulation Parameters

### Arena Configuration
```rust
const CAPACITY: usize = 2048;
const CELL_CAPACITY: usize = 32;
```

### SimConfig Defaults
```rust
carrying_capacity: 800,
reproduction_threshold: 120.0,
reproduction_cost: 40.0,
max_age: 2000.0,
base_mortality: 0.00002,
starvation_threshold: 10.0,
```

## Visual Elements

### Boid Rendering
- Color: HSL based on role + speed + metabolism
- Size: Scaled by `genome.size`
- Opacity: Based on energy level
- Direction indicator (triangle pointing velocity)

### Food Sources
- Green circles with transparency based on fullness
- Pulsing effect when being consumed
- Regeneration visualized by growing radius

### UI Elements
- Population counts by role
- Generation counter
- Season indicator
- Diversity score
- FPS counter

## Controls

| Key | Action |
|-----|--------|
| Space | Pause/resume |
| R | Reset simulation |
| F | Spawn food source at cursor |
| P | Spawn predator zone |
| +/- | Adjust time scale |

## Common Tasks

### Tuning population dynamics
1. Adjust `SimConfig` values in `main.rs`
2. Key levers: `carrying_capacity`, `reproduction_threshold`, `base_mortality`
3. Test with diversity score - aim for 0.5-0.8

### Adding a new boid behavior
1. Add `BoidState` variant in `core/src/lib.rs`
2. Implement force logic in `compute_flocking_forces`
3. Add state transition in `update_states`
4. Update color/visual if behavior is visible

### Changing rendering style
1. Modify draw functions in `main.rs`
2. Use canvas 2D API via `web_sys`
3. Consider performance (called per-boid per-frame)

## Performance Notes

- Target: 60 FPS with 1000+ boids
- Bottleneck: neighbor queries (spatial grid helps)
- Avoid: per-frame allocations, complex per-boid rendering
- Profile with browser dev tools

## Testing

Visual testing via Playwright:
```bash
npx playwright test tests/toofoo.spec.ts
```

Core logic tested in `core`:
```bash
cargo test -p core
```

Manual testing checklist:
- [ ] Populations stabilize (no extinction/explosion)
- [ ] Predators hunt successfully
- [ ] Food sources regenerate
- [ ] Seasons affect food availability
- [ ] Evolution visible over generations
