# DNA - Foundation Layer

Shared algorithms, physics, and math for all antimony-labs projects.
Zero-allocation, cache-friendly design.

## Build & Test

```bash
cargo check -p dna
cargo test -p dna
cargo doc -p dna --open
```

## Architecture

DNA is the foundation layer. All CORE engines depend on DNA. Projects depend on CORE engines.

```
DNA (foundation)
 └── CORE engines (domain-specific)
      └── Projects (applications)
```

## Module Overview

```
DNA/src/
├── lib.rs              # Public API exports
│
├── physics/            # Physics foundations
│   ├── mod.rs
│   ├── electromagnetics/
│   │   ├── mod.rs
│   │   └── lumped.rs   # R, L, C, Diode, OpAmp, transistors
│   └── solvers/
│       ├── mod.rs
│       ├── rk45.rs     # Runge-Kutta 4/5 integrator
│       └── filters.rs  # EKF, trajectory smoothing
│
├── cad/                # B-Rep CAD kernel
│   ├── mod.rs
│   ├── geometry.rs     # Point3, Vector3, Plane, Transform3
│   ├── topology.rs     # Vertex, Edge, Face, Loop, Shell, Solid
│   └── primitives.rs   # make_box, make_cylinder, make_sphere, make_cone
│
├── pll/                # Phase-Locked Loop components
│   ├── mod.rs
│   ├── components.rs   # VCO, PFD, ChargePump, Divider
│   ├── loop_filter.rs  # ActiveLoopFilter, PassiveLoopFilter
│   ├── stability.rs    # phase_margin, loop_bandwidth
│   ├── integer_n.rs    # Integer-N synthesizer
│   └── transient.rs    # Transient simulation
│
├── sim/                # Simulation algorithms
│   ├── mod.rs
│   ├── boid_arena.rs   # Fixed-capacity entity storage (SoA)
│   ├── spatial_grid.rs # O(1) neighbor queries
│   └── state_machine.rs # Behavior state transitions
│
├── export/             # File format exporters
│   ├── mod.rs
│   ├── pdf.rs          # PDF generation
│   └── gerber.rs       # Gerber X2 format
│
├── autocrate/          # Shipping crate design
│   └── mod.rs          # CrateGeometry, calculate_boards
│
├── heliosphere.rs      # Heliospheric boundary models
├── heliosphere_model.rs # Parker spiral, termination shock
├── solar_wind.rs       # Solar wind particles
├── coordinates.rs      # Coordinate transforms
└── ... (other modules)
```

## Key Types

### Physics: Lumped Elements

```rust
use dna::physics::electromagnetics::lumped::*;

let r = Resistor::new(1000.0);           // 1kΩ
let c = Capacitor::new(100e-12);         // 100pF
let l = Inductor::new(10e-6);            // 10µH
let d = Diode::ideal();
let op = OpAmp::ideal();
```

### Physics: Solvers

```rust
use dna::physics::solvers::rk45::*;
use dna::physics::solvers::filters::EKF;

// RK45 integration
let result = rk45_integrate(state, t0, t1, dt, |s, t| derivatives);

// Extended Kalman Filter
let ekf = EKF::new(4, 2);  // 4 states, 2 measurements
```

### CAD: Geometry & Topology

```rust
use dna::cad::*;

let solid = primitives::make_box(100.0, 50.0, 25.0);
let cylinder = primitives::make_cylinder(10.0, 50.0, 32);

let builder = SolidBuilder::from_box(100.0, 50.0, 25.0)
    .translate(10.0, 0.0, 0.0)
    .rotate_z(0.5)
    .build();
```

### PLL: Components

```rust
use dna::pll::*;

let vco = VCO::new(1e9, 50e6);           // 1GHz center, 50MHz/V
let pfd = PFD::new();
let cp = ChargePump::new(1e-3);          // 1mA
let filter = ActiveLoopFilter::new(1e3, 1e-9, 10e3);
```

### Simulation: BoidArena

```rust
use dna::sim::{BoidArena, SpatialGrid, Genome};

let mut arena = BoidArena::<1024>::new();
let handle = arena.spawn(pos, vel, Genome::random(&mut rng));

let mut grid = SpatialGrid::<32>::new(800.0, 600.0, 50.0);
grid.build(&arena);

let mut neighbors = [0u16; 64];
let count = grid.query_neighbors(pos, 100.0, &arena, None, &mut neighbors);
```

## Testing

```bash
# All DNA tests
cargo test -p dna

# Specific module
cargo test -p dna -- pll
cargo test -p dna -- cad
cargo test -p dna -- physics

# With output
cargo test -p dna -- --nocapture
```

## Performance Guidelines

1. **No allocations in hot paths**: Use pre-sized buffers
2. **Stack-allocated arrays**: `let mut neighbors = [0u16; 64];`
3. **Use `#[inline]` for small per-entity functions**
4. **Guard against NaN**: Check vector length before normalize
5. **Prefer f32 over f64** for simulation (matches GPU)

## Deprecated Modules

These modules are deprecated but still available for backward compatibility:

```rust
// Old path (deprecated)
use dna::spice::*;
// New path
use dna::physics::electromagnetics::lumped::*;

// Old path (deprecated)
use dna::ekf::EKF;
// New path
use dna::physics::solvers::filters::EKF;
```

## Common Tasks

### Adding a new physics component
1. Add struct to `physics/electromagnetics/lumped.rs`
2. Implement `Component` trait if applicable
3. Add tests
4. Export from `physics/electromagnetics/mod.rs`

### Adding a CAD primitive
1. Add function to `cad/primitives.rs`
2. Create proper topology (vertices, edges, faces)
3. Add tests
4. Export from `cad/mod.rs`

### Adding a PLL component
1. Add to appropriate file in `pll/`
2. Implement component behavior
3. Add tests
4. Export from `pll/mod.rs`
