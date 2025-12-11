//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: point.rs | DNA/src/world/cca/point.rs
//! PURPOSE: Conformal point (5D null vector) in Conformal Geometric Algebra
//! CREATED: 2025-12-09
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! # Conformal Point
//!
//! A point in 3D Euclidean space embedded into 5D conformal space as a null vector.
//!
//! ## Embedding
//!
//! For a point p = (x, y, z) ∈ ℝ³:
//! ```text
//! P = x·e₁ + y·e₂ + z·e₃ + ½|p|²·e∞ + eₒ
//! ```
//!
//! Where:
//! - e₁, e₂, e₃ are the standard 3D basis vectors
//! - e∞ = e₊ + e₋ is the point at infinity
//! - eₒ = ½(e₋ - e₊) is the origin
//!
//! ## Key Properties
//!
//! - P · P = 0 (null vector - lies on the null cone)
//! - P · Q = -½|p - q|² (inner product gives squared distance)
//! - Transformations via sandwich product: P' = M·P·M̃
//!
//! ═══════════════════════════════════════════════════════════════════════════════

use glam::DVec3;
use std::ops::{Add, Mul, Sub};

/// A point in Conformal Geometric Algebra (5D null vector)
///
/// Represents a 3D Euclidean point embedded in 5D conformal space.
/// The embedding preserves distances (inner product gives squared distance)
/// and allows all rigid transformations to be represented as rotors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConformalPoint {
    /// Coefficients in the basis [e₁, e₂, e₃, e₊, e₋]
    /// where e₊² = +1 and e₋² = -1
    pub coords: [f64; 5],
}

impl ConformalPoint {
    /// Create a conformal point from raw coefficients
    ///
    /// WARNING: This does not ensure the point lies on the null cone.
    /// Use `from_euclidean` for proper embedding.
    #[inline]
    pub const fn from_raw(coords: [f64; 5]) -> Self {
        Self { coords }
    }

    /// Embed a 3D Euclidean point into conformal space
    ///
    /// The embedding formula is:
    /// P = x·e₁ + y·e₂ + z·e₃ + ½|p|²·e∞ + eₒ
    ///
    /// Using e∞ = e₊ + e₋ and eₒ = ½(e₋ - e₊):
    /// - e₊ coefficient = ½|p|² - ½ = ½(|p|² - 1)
    /// - e₋ coefficient = ½|p|² + ½ = ½(|p|² + 1)
    #[inline]
    pub fn from_euclidean(p: DVec3) -> Self {
        let x2 = p.length_squared();
        Self {
            coords: [
                p.x,
                p.y,
                p.z,
                0.5 * (x2 - 1.0), // e₊ coefficient
                0.5 * (x2 + 1.0), // e₋ coefficient
            ],
        }
    }

    /// Create from x, y, z coordinates
    #[inline]
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self::from_euclidean(DVec3::new(x, y, z))
    }

    /// Create the origin point (0, 0, 0)
    #[inline]
    pub fn origin() -> Self {
        Self {
            coords: [0.0, 0.0, 0.0, -0.5, 0.5],
        }
    }

    /// Create the point at infinity
    ///
    /// e∞ = e₊ + e₋ represents the point at infinity in projective sense
    #[inline]
    pub fn infinity() -> Self {
        Self {
            coords: [0.0, 0.0, 0.0, 1.0, 1.0],
        }
    }

    /// Extract the 3D Euclidean point (dehomogenize)
    ///
    /// For a conformal point P = x·e₁ + y·e₂ + z·e₃ + α·e₊ + β·e₋
    /// the Euclidean point is (x, y, z) / w where w = -P · e∞ = β - α
    #[inline]
    pub fn to_euclidean(&self) -> DVec3 {
        // w = coefficient of eₒ in the dual representation
        // For standard embedding: w = e₋ - e₊ coefficient = coords[4] - coords[3]
        let w = self.coords[4] - self.coords[3];

        if w.abs() < 1e-15 {
            // Point at infinity - return a large but finite vector
            return DVec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        }

        DVec3::new(self.coords[0] / w, self.coords[1] / w, self.coords[2] / w)
    }

    /// Get the x coordinate (e₁ component)
    #[inline]
    pub fn x(&self) -> f64 {
        let w = self.coords[4] - self.coords[3];
        if w.abs() < 1e-15 {
            f64::INFINITY
        } else {
            self.coords[0] / w
        }
    }

    /// Get the y coordinate (e₂ component)
    #[inline]
    pub fn y(&self) -> f64 {
        let w = self.coords[4] - self.coords[3];
        if w.abs() < 1e-15 {
            f64::INFINITY
        } else {
            self.coords[1] / w
        }
    }

    /// Get the z coordinate (e₃ component)
    #[inline]
    pub fn z(&self) -> f64 {
        let w = self.coords[4] - self.coords[3];
        if w.abs() < 1e-15 {
            f64::INFINITY
        } else {
            self.coords[2] / w
        }
    }

    /// Inner product in conformal space (Minkowski metric ℝ⁴'¹)
    ///
    /// The signature is (+,+,+,+,-) meaning:
    /// P · Q = p₁q₁ + p₂q₂ + p₃q₃ + p₄q₄ - p₅q₅
    ///
    /// For properly embedded points: P · Q = -½|p - q|²
    #[inline]
    pub fn inner(&self, other: &Self) -> f64 {
        self.coords[0] * other.coords[0]
            + self.coords[1] * other.coords[1]
            + self.coords[2] * other.coords[2]
            + self.coords[3] * other.coords[3]
            - self.coords[4] * other.coords[4] // Negative signature for e₋
    }

    /// Squared Euclidean distance to another point
    ///
    /// Uses the conformal property: |p - q|² = -2(P · Q)
    #[inline]
    pub fn distance_squared(&self, other: &Self) -> f64 {
        -2.0 * self.inner(other)
    }

    /// Euclidean distance to another point
    #[inline]
    pub fn distance(&self, other: &Self) -> f64 {
        self.distance_squared(other).max(0.0).sqrt()
    }

    /// Check if this is a valid null vector (lies on null cone)
    ///
    /// A properly embedded conformal point satisfies P · P = 0
    #[inline]
    pub fn is_null(&self) -> bool {
        self.inner(self).abs() < 1e-10
    }

    /// Check if this represents a point at infinity
    #[inline]
    pub fn is_infinite(&self) -> bool {
        let w = self.coords[4] - self.coords[3];
        w.abs() < 1e-10
    }

    /// Normalize the conformal point (ensure w = 1)
    ///
    /// This doesn't change the represented Euclidean point,
    /// just the homogeneous representation.
    #[inline]
    pub fn normalize(&self) -> Self {
        let w = self.coords[4] - self.coords[3];
        if w.abs() < 1e-15 {
            return *self; // Can't normalize point at infinity
        }

        Self {
            coords: [
                self.coords[0] / w,
                self.coords[1] / w,
                self.coords[2] / w,
                self.coords[3] / w,
                self.coords[4] / w,
            ],
        }
    }

    /// Scale the conformal point (for weighted points)
    #[inline]
    pub fn scale(&self, s: f64) -> Self {
        Self {
            coords: [
                self.coords[0] * s,
                self.coords[1] * s,
                self.coords[2] * s,
                self.coords[3] * s,
                self.coords[4] * s,
            ],
        }
    }
}

impl Default for ConformalPoint {
    fn default() -> Self {
        Self::origin()
    }
}

impl Add for ConformalPoint {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        Self {
            coords: [
                self.coords[0] + other.coords[0],
                self.coords[1] + other.coords[1],
                self.coords[2] + other.coords[2],
                self.coords[3] + other.coords[3],
                self.coords[4] + other.coords[4],
            ],
        }
    }
}

impl Sub for ConformalPoint {
    type Output = Self;

    #[inline]
    fn sub(self, other: Self) -> Self {
        Self {
            coords: [
                self.coords[0] - other.coords[0],
                self.coords[1] - other.coords[1],
                self.coords[2] - other.coords[2],
                self.coords[3] - other.coords[3],
                self.coords[4] - other.coords[4],
            ],
        }
    }
}

impl Mul<f64> for ConformalPoint {
    type Output = Self;

    #[inline]
    fn mul(self, s: f64) -> Self {
        self.scale(s)
    }
}

impl From<DVec3> for ConformalPoint {
    #[inline]
    fn from(p: DVec3) -> Self {
        Self::from_euclidean(p)
    }
}

impl From<(f64, f64, f64)> for ConformalPoint {
    #[inline]
    fn from((x, y, z): (f64, f64, f64)) -> Self {
        Self::new(x, y, z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-10;

    #[test]
    fn test_embedding_roundtrip() {
        let p = DVec3::new(1.0, 2.0, 3.0);
        let cp = ConformalPoint::from_euclidean(p);
        let back = cp.to_euclidean();

        assert!((p - back).length() < EPSILON);
    }

    #[test]
    fn test_null_vector() {
        // Properly embedded points should be null vectors
        let cp = ConformalPoint::new(3.0, 4.0, 5.0);
        assert!(cp.is_null(), "Embedded point should be null vector");
    }

    #[test]
    fn test_distance_via_inner_product() {
        let p1 = ConformalPoint::new(0.0, 0.0, 0.0);
        let p2 = ConformalPoint::new(3.0, 4.0, 0.0);

        let dist = p1.distance(&p2);
        assert!(
            (dist - 5.0).abs() < EPSILON,
            "Distance should be 5, got {}",
            dist
        );
    }

    #[test]
    fn test_origin() {
        let origin = ConformalPoint::origin();
        let euclidean = origin.to_euclidean();

        assert!(euclidean.length() < EPSILON);
        assert!(origin.is_null());
    }

    #[test]
    fn test_infinity() {
        let inf = ConformalPoint::infinity();
        assert!(inf.is_infinite());
    }

    #[test]
    fn test_multiple_distances() {
        // Test various distances to ensure conformal property holds
        let test_cases = [
            ((0.0, 0.0, 0.0), (1.0, 0.0, 0.0), 1.0),
            ((0.0, 0.0, 0.0), (0.0, 1.0, 0.0), 1.0),
            ((1.0, 1.0, 1.0), (2.0, 2.0, 2.0), 3.0_f64.sqrt()),
            ((0.0, 0.0, 0.0), (3.0, 4.0, 0.0), 5.0),
        ];

        for ((x1, y1, z1), (x2, y2, z2), expected) in test_cases {
            let p1 = ConformalPoint::new(x1, y1, z1);
            let p2 = ConformalPoint::new(x2, y2, z2);
            let dist = p1.distance(&p2);

            assert!(
                (dist - expected).abs() < EPSILON,
                "Distance from ({},{},{}) to ({},{},{}) should be {}, got {}",
                x1,
                y1,
                z1,
                x2,
                y2,
                z2,
                expected,
                dist
            );
        }
    }

    #[test]
    fn test_normalization() {
        let p = ConformalPoint::new(1.0, 2.0, 3.0);
        let scaled = p.scale(5.0);
        let normalized = scaled.normalize();

        // Should represent same point
        let p_euclidean = p.to_euclidean();
        let normalized_euclidean = normalized.to_euclidean();

        assert!((p_euclidean - normalized_euclidean).length() < EPSILON);
    }
}
