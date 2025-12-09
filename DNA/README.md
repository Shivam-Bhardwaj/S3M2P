# S3M2P DNA Core

The **DNA** crate is the core algorithmic and simulation engine for the S3M2P project. It provides the fundamental logic for boid navigation, ecosystem simulation, wave field dynamics, and export capabilities.

## Architecture

The crate is organized into:
*   **Domain-Specific Modules**: `heliosphere`, `solar_wind`, `sim` (specialized simulations like Chladni plates).
*   **Shared Utilities**: `interaction`, `zones`, `statistics`, `color`.
*   **Math & Physics**: `mat2`, `spatial`, `pathfinding`.

## Key Features

*   **Boids Simulation**: Flocking behavior with predator/prey dynamics.
*   **Pathfinding**: Optimized A* implementation on grid maps.
*   **Wave Fields**: Chladni pattern generation and wave propagation.
*   **Heliosphere**: Modeling of solar wind termination shocks.

## Testing & Coverage

Run tests with:
```bash
cargo test -p dna
```

Generate coverage:
```bash
cargo llvm-cov -p dna
```
