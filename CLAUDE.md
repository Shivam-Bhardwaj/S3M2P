# S3M2P - Simulation Systems Monorepo

This is a Rust/WASM monorepo containing simulation engines and visualizations.

## Quick Reference

| Command | Description |
|---------|-------------|
| `cargo check --workspace` | Type check all crates |
| `cargo test --workspace` | Run all tests |
| `trunk build helios/index.html` | Build helios WASM |
| `trunk build too.foo/index.html` | Build too.foo WASM |
| `trunk serve helios/index.html` | Dev server for helios |
| `npx playwright test` | Visual regression tests |
| `./scripts/worktree.sh create <issue>` | Create worktree for issue |

## Architecture

```
S3M2P/
  core/           # Shared simulation engine (Rust library)
  helios/         # Solar system visualization (Rust/WASM)
  too.foo/        # Boid ecosystem visualization (Rust/WASM)
  storage-server/ # Backend persistence (Rust)
  simulation-cli/ # CLI tools (Rust)
  PROJECT_N/      # In development
```

## Project Dependencies

```
core <── helios
     <── too.foo
     <── simulation-cli
storage-server (standalone)
```

## Core Concepts

### BoidArena (core)
Fixed-capacity, zero-allocation entity storage using Structure of Arrays (SoA) layout.
- `BoidHandle`: Generational index for safe entity references
- O(1) spawn/kill operations via free list
- Pre-allocated scratch buffers for per-frame computations

### SpatialGrid (core)
Spatial partitioning for O(1) neighbor queries.
- Fixed-size cells, no per-cell allocations
- `query_neighbors()` writes to caller-provided buffer

### State Machine (core)
Boid behavior states: `Wander`, `Forage`, `Hunt`, `Flee`, `Reproduce`, `Dead`
- State transitions based on energy, threats, and neighbors
- Different flocking forces per state

## Development Workflow

### Starting Work on an Issue

1. Create issue using GitHub templates (enforces project labels)
2. Use `/work <issue-number>` command in Claude Code
3. This creates a worktree and branch automatically
4. Work in isolation, then PR back to main

### Validation Before Commit

Use `/validate` command to run:
- `cargo check` for affected crates
- `cargo test` for test crates
- `trunk build` for WASM crates
- `playwright test` if UI changed

### Creating PRs

Use `/pr` command to:
- Generate PR from current branch
- Link to issue
- Include test plan

## Code Style

- Rust 2021 edition
- No heap allocations in hot paths (simulation loop)
- Use `#[inline]` for small functions called per-entity
- Wrap coordinates at world boundaries (toroidal topology)
- Energy clamped to [0, 200], metabolism affects drain rate

## Testing

- Unit tests in `core/src/*.rs` (run with `cargo test -p core`)
- Visual regression tests in `tests/` (Playwright)
- Snapshot tests for canvas rendering

## Common Pitfalls

1. **Zero-length vectors**: Always check `length() > epsilon` before normalizing
2. **WASM bindings**: Use `wasm-bindgen = "=0.2.93"` (pinned version)
3. **getrandom**: Requires `features = ["js"]` for WASM target
4. **Canvas coordinates**: Y-axis increases downward

## File Patterns

- `src/lib.rs` - Public API exports
- `src/main.rs` - WASM entry point (projects)
- `src/render.rs` - Canvas rendering code
- `src/simulation.rs` - Per-frame update logic
