# Chladni - Wave Pattern Visualization

Rust/WASM port of Chladni plate pattern simulation with particle-based sand visualization.

## Build & Run

```bash
trunk serve chladni/index.html --open
trunk build --release chladni/index.html
```

## Architecture

```
chladni/
  src/
    lib.rs       # Main simulation state, particle physics
    wave.rs      # Wave equation solver, eigenmode calculation
    renderer.rs  # WebGL2 rendering
  index.html     # Entry point with mode controls
```

## Core Types

### PlateMode
Defines vibration pattern with (m, n) mode numbers:
- m = horizontal mode number
- n = vertical mode number
- Higher numbers = more complex patterns

### WaveSimulation
2D wave field on a grid:
- Amplitude field (height at each point)
- Velocity field (rate of change)
- Energy density for visualization

### Particle
Sand particles that settle at nodal lines:
- Position and velocity
- Move based on wave gradient
- Accumulate at low-amplitude regions

## Key Algorithms

### Chladni Eigenmode
Standing wave pattern for square plate:
```
A_mn(x,y) = sin(m*π*x/L) * sin(n*π*y/L)
```

### Particle Movement
Particles move toward nodal lines (amplitude minima):
```
force = -∇(amplitude²) * scale
```

## Implementation Status

- [x] Wave eigenmode calculation
- [x] Particle physics
- [x] Basic WebGL2 rendering
- [ ] Dynamic wave simulation (not just eigenmodes)
- [ ] Multiple visualization modes
- [ ] Sound integration (frequency playback)
- [ ] Touch/click excitation

## Original Source

C++/CUDA: https://github.com/Shivam-Bhardwaj/chladni-realistic-rendering

Key concepts ported:
- Wave equation solver → `wave.rs`
- Particle system → `lib.rs`
- Visualization modes → `renderer.rs`
