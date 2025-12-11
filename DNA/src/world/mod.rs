//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: mod.rs | DNA/src/world/mod.rs
//! PURPOSE: Module exports for world
//! MODIFIED: 2025-12-09
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════

//!
//! WORLD defines WHERE things exist in space:
//! - coordinates/  - Cartesian, spherical (cylindrical, polar - future)
//! - transforms/   - Astronomical coordinate transforms
//! - topology/     - Toroidal wrap-around boundaries
//! - grid/         - Spatial grids (future)
//! - units.rs      - Physical units and conversions
//! - cca/          - Conformal Celestial Algebra (CGA + SE(3) Lie groups)
//!
//! ═══════════════════════════════════════════════════════════════════════════════

// ─────────────────────────────────────────────────────────────────────────────────
// SUBMODULES
// ─────────────────────────────────────────────────────────────────────────────────

pub mod cca;
pub mod coordinates;
pub mod grid;
pub mod stars;
pub mod topology;
pub mod transforms;
pub mod units;
