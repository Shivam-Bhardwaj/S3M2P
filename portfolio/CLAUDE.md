# Portfolio - Interactive Robotics Demos

Personal website with interactive robotics/simulation demonstrations.

## Build & Run

```bash
trunk serve portfolio/index.html --open
trunk build --release portfolio/index.html
```

## Architecture

```
portfolio/
  src/
    lib.rs       # WASM entry point
  index.html     # Entry point
  style.css      # Styles
```

## Planned Demos

1. **Boids**: Flocking simulation with EKF-smoothed rendering
2. **A* Pathfinding**: Interactive grid-based pathfinding
3. **EKF**: Kalman filter visualization (noisy vs filtered)
4. **About**: Personal info page

## Dependencies from Core

```rust
use core::{EKF, Mat2};           // State estimation
use core::{GridMap, astar};       // Pathfinding
```

## Implementation Status

This is a scaffold. Implementation tracked via issues:
- [ ] Boid demo implementation
- [ ] Pathfinding demo implementation
- [ ] EKF demo implementation
- [ ] Event handling and animation loop
- [ ] Mobile touch support

## Porting from shivambhardwaj.com

Original source: https://github.com/Shivam-Bhardwaj/shivambhardwaj.com

Key algorithms from `robotics_lib/` have been merged into `core`:
- EKF (Extended Kalman Filter)
- Mat2 (2x2 matrix operations)
- Pathfinding (A* with grid maps)
