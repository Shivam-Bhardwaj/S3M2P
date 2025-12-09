# Kilo Code Context & Instructions

## Project Overview
**Antimony Labs (S3M2P)** is an AI-native development platform and monorepo for simulations, visualizations, and full product development.
- **URL**: [too.foo](https://too.foo)
- **Stack**: Rust (Stable), WebAssembly (WASM), Trunk, Cloudflare Pages.
- **Philosophy**: "Let AI Design, Humans Build". Minimal dependencies (write from scratch), mobile-first, DNA-driven (TOML config).

## Architecture
The project follows a strict three-tier dependency flow:
1.  **DNA/**: Foundation layer. Shared algorithms, physics, math, and simulation engines.
2.  **CORE/**: Domain-specific engines (e.g., `SIMULATION_ENGINE`, `SPICE_ENGINE`, `CAD_ENGINE`).
3.  **Projects**: User-facing applications (e.g., `WELCOME`, `HELIOS`, `TOOLS/*`).
    - **Dependency Rule**: `DNA` ← `CORE` ← `Projects`. Never the reverse.

## Key Directories
- `DNA/`: Core logic (Boids, Physics, Math).
- `CORE/`: Specialized engines.
- `WELCOME/`: Landing page (too.foo).
- `HELIOS/`: Solar system simulation.
- `TOOLS/`: Utilities (PLL, Autocrate, Sensors).
- `SCRIPTS/`: Automation scripts.

## Development Workflow
1.  **Worktrees**: Always work in a dedicated worktree for an issue.
    - `work <issue_id>`: Creates `~/worktrees/issue-<id>` and sets up the environment.
2.  **Dev Server**:
    - `run <project>`: Starts the dev server for a specific project (e.g., `run helios`).
    - Auto-assigns ports (Welcome: 8080, Helios: 8081, etc.).
3.  **Validation**:
    - `./SCRIPTS/validate.sh`: Runs `cargo check`, `cargo test`, `clippy`, and `trunk build`.
4.  **Deployment**:
    - Push to `preview/issue-<id>` for preview deployment.
    - Merge to `main` for production.

## Common Commands
| Command | Description |
|---------|-------------|
| `run <project>` | Start dev server (e.g., `run welcome`) |
| `work <issue_id>` | Create worktree and start work on an issue |
| `cargo check --workspace` | Check all crates |
| `cargo test --workspace` | Run all tests |
| `./SCRIPTS/validate.sh` | Run full validation suite |
| `./SCRIPTS/deploy.sh <proj> --publish` | Deploy to Cloudflare |

## Coding Standards
- **Rust**: 2021 edition. No heap allocations in hot paths. Use `#[inline]` for small functions.
- **WASM**: Target `wasm32-unknown-unknown`.
- **DNA Code**: Use TOML for configuration (Parts, Machines, PCBs).
- **Style**: Follow `rustfmt` and `clippy`.

## AI Instructions
- **Read-Only**: You generally do not need to read `CLAUDE.md` or `FIELD_MANUAL.md` as this file summarizes them.
- **Context**: Always check which directory you are in. If in a worktree, you are scoped to that issue.
- **Edits**: Prefer `apply_diff` for surgical changes. Use `write_to_file` for new files.
- **Testing**: Always validate changes with `cargo check` and `cargo test` before finishing.