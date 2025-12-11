//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: mod.rs | DNA/src/world/cca/mod.rs
//! PURPOSE: Conformal Celestial Algebra (CCA) - Novel coordinate framework
//! CREATED: 2025-12-09
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! # Conformal Celestial Algebra (CCA)
//!
//! A novel coordinate system framework combining:
//! - **Conformal Geometric Algebra (CGA)** - 5D representation where all rigid
//!   transformations are rotors (sandwich products)
//! - **SE(3) Lie Groups** - Proper manifold structure for frame transformations
//!   with uncertainty propagation via adjoint action
//! - **HEALPix-inspired indexing** - Extended to conformal space for spatial queries
//!
//! ## Key Innovations
//!
//! 1. Keplerian orbits as conformal conics (circles/ellipses are blades)
//! 2. Frame transformations are rotors (single representation for all transforms)
//! 3. Covariance propagation in Lie algebra (proper uncertainty handling)
//! 4. Learning in tangent space (SE(3)-equivariant neural networks)
//!
//! ## Mathematical Foundation
//!
//! We embed 3D Euclidean space into 5D conformal space:
//! ```text
//! e₁, e₂, e₃    : standard 3D basis
//! e₊, e₋        : extra dimensions
//! eₒ = ½(e₋ - e₊)  : origin
//! e∞ = e₋ + e₊     : point at infinity
//!
//! Point x ∈ ℝ³  →  X = x + ½|x|²e∞ + eₒ  (null vector in ℝ⁴'¹)
//! ```
//!
//! ## References
//!
//! - Hestenes, D. "Celestial Mechanics with Geometric Algebra" (1983)
//! - Dorst, L. "Geometric Algebra for Computer Science" (2007)
//! - Selig, J.M. "Lie Groups and Lie Algebras in Robotics" (2004)
//!
//! ═══════════════════════════════════════════════════════════════════════════════

pub mod blade;
pub mod epoch;
pub mod frame_graph;
pub mod motor;
pub mod point;
pub mod se3;

// Re-exports for convenient access
pub use blade::{Circle, Line, Plane, Sphere};
pub use epoch::{Epoch, TimeScale};
pub use frame_graph::{CelestialBody, FrameDef, FrameGraph, FrameId};
pub use motor::Motor;
pub use point::ConformalPoint;
pub use se3::{Se3, SO3};

/// Basis vectors for Conformal Geometric Algebra (CGA)
///
/// The conformal model uses ℝ⁴'¹ (4 positive, 1 negative signature)
pub mod basis {
    /// Standard 3D basis index for e₁
    pub const E1: usize = 0;
    /// Standard 3D basis index for e₂
    pub const E2: usize = 1;
    /// Standard 3D basis index for e₃
    pub const E3: usize = 2;
    /// Extra dimension e₊ (positive signature)
    pub const EP: usize = 3;
    /// Extra dimension e₋ (negative signature)
    pub const EM: usize = 4;

    /// Origin point: eₒ = ½(e₋ - e₊)
    #[inline]
    pub fn origin() -> [f64; 5] {
        [0.0, 0.0, 0.0, -0.5, 0.5]
    }

    /// Point at infinity: e∞ = e₋ + e₊
    #[inline]
    pub fn infinity() -> [f64; 5] {
        [0.0, 0.0, 0.0, 1.0, 1.0]
    }
}

/// Physical and astronomical constants
pub mod constants {
    /// Astronomical Unit in kilometers
    pub const AU_KM: f64 = 149_597_870.7;
    /// Solar radius in kilometers
    pub const SOLAR_RADIUS_KM: f64 = 695_700.0;
    /// Earth radius in kilometers
    pub const EARTH_RADIUS_KM: f64 = 6_371.0;
    /// J2000.0 epoch as Julian Date
    pub const J2000_EPOCH_JD: f64 = 2_451_545.0;
    /// Seconds per Julian day
    pub const SECONDS_PER_DAY: f64 = 86_400.0;
    /// Days per Julian year
    pub const DAYS_PER_YEAR: f64 = 365.25;
    /// Gravitational parameter of Sun (km³/s²)
    pub const GM_SUN: f64 = 1.327_124_400_18e11;
    /// Gravitational parameter of Earth (km³/s²)
    pub const GM_EARTH: f64 = 3.986_004_418e5;
}

/// Reference frame identifiers
///
/// These are compile-time constants for type-safe frame handling
pub mod frames {
    /// International Celestial Reference Frame (barycentric)
    pub const ICRF: u32 = 0;
    /// Heliocentric Inertial (J2000 ecliptic, Sun-centered)
    pub const HCI: u32 = 1;
    /// Heliocentric Earth Ecliptic (X toward Earth)
    pub const HEE: u32 = 2;
    /// Heliocentric Aries Ecliptic
    pub const HAE: u32 = 3;
    /// Geocentric Inertial (J2000 equator)
    pub const GCI: u32 = 4;
    /// Geocentric Solar Ecliptic (X toward Sun)
    pub const GSE: u32 = 5;
    /// Geocentric Solar Magnetospheric
    pub const GSM: u32 = 6;
    /// Radial-Tangential-Normal (spacecraft local)
    pub const RTN: u32 = 7;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basis_vectors() {
        let origin = basis::origin();
        let infinity = basis::infinity();

        // Check normalization: eₒ · eₒ = 0 (null vector)
        // In conformal space, origin and infinity are null vectors
        // eₒ · e∞ = -1 (they are normalized against each other)

        // Inner product in ℝ⁴'¹: positive for first 4, negative for last
        let eo_dot_einf = origin[0] * infinity[0]
            + origin[1] * infinity[1]
            + origin[2] * infinity[2]
            + origin[3] * infinity[3]
            - origin[4] * infinity[4]; // Note: e₋ has negative signature

        assert!(
            (eo_dot_einf + 1.0).abs() < 1e-10,
            "eₒ · e∞ should be -1, got {}",
            eo_dot_einf
        );
    }
}
