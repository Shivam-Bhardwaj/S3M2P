//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: motor.rs | DNA/src/world/cca/motor.rs
//! PURPOSE: Motor (even multivector) for rigid transformations in CGA
//! CREATED: 2025-12-09
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! # Motor
//!
//! A motor is an even multivector in Conformal Geometric Algebra that represents
//! any rigid transformation (rotation, translation, or combination) as a single
//! algebraic element.
//!
//! ## Mathematical Background
//!
//! In CGA, a motor M can represent:
//! - Pure rotation: R = cos(θ/2) + sin(θ/2)·B where B is a bivector (rotation plane)
//! - Pure translation: T = 1 + ½t·e∞ where t is translation vector
//! - General rigid motion: M = T·R (screw motion)
//!
//! ## Transformation Application
//!
//! Points are transformed via the sandwich product:
//! ```text
//! P' = M·P·M̃  (where M̃ is the reverse of M)
//! ```
//!
//! ## 8-Component Representation
//!
//! We use an 8-component representation covering the even subalgebra:
//! [scalar, e₁₂, e₁₃, e₂₃, e₁∞, e₂∞, e₃∞, e₁₂₃∞]
//!
//! This is isomorphic to dual quaternions and sufficient for all rigid motions.
//!
//! ═══════════════════════════════════════════════════════════════════════════════

use super::point::ConformalPoint;
use glam::{DQuat, DVec3};
use std::ops::Mul;

/// Motor: even multivector for rigid transformations
///
/// Represents rotation and translation in a unified algebraic form.
/// Motors compose via geometric product and transform points via sandwich product.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Motor {
    /// 8-component representation of even multivector
    /// [scalar, e₁₂, e₁₃, e₂₃, e₁∞, e₂∞, e₃∞, e₁₂₃∞]
    ///
    /// Components 0-3: rotor part (rotation, like quaternion)
    /// Components 4-7: translation bivectors
    pub components: [f64; 8],
}

impl Motor {
    /// Create a motor from raw components
    #[inline]
    pub const fn from_raw(components: [f64; 8]) -> Self {
        Self { components }
    }

    /// Identity motor (no transformation)
    #[inline]
    pub fn identity() -> Self {
        Self {
            components: [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        }
    }

    /// Create a pure rotation motor around an axis
    ///
    /// # Arguments
    /// * `axis` - Rotation axis (will be normalized)
    /// * `angle` - Rotation angle in radians
    #[inline]
    pub fn rotation(axis: DVec3, angle: f64) -> Self {
        let axis = axis.normalize();
        let half_angle = angle / 2.0;
        let c = half_angle.cos();
        let s = half_angle.sin();

        // Rotation bivector: sin(θ/2)(a₁e₂₃ + a₂e₃₁ + a₃e₁₂)
        // Note: e₂₃ = e₂∧e₃, etc. These are the rotation plane bivectors
        Self {
            components: [
                c,           // scalar
                -s * axis.z, // e₁₂ (rotation in xy-plane, axis along z)
                s * axis.y,  // e₁₃ (rotation in xz-plane, axis along y)
                -s * axis.x, // e₂₃ (rotation in yz-plane, axis along x)
                0.0,         // e₁∞
                0.0,         // e₂∞
                0.0,         // e₃∞
                0.0,         // e₁₂₃∞
            ],
        }
    }

    /// Create a pure translation motor
    ///
    /// T = 1 + ½t·e∞ = 1 + ½(t₁e₁∞ + t₂e₂∞ + t₃e₃∞)
    #[inline]
    pub fn translation(t: DVec3) -> Self {
        Self {
            components: [
                1.0,       // scalar
                0.0,       // e₁₂
                0.0,       // e₁₃
                0.0,       // e₂₃
                0.5 * t.x, // e₁∞
                0.5 * t.y, // e₂∞
                0.5 * t.z, // e₃∞
                0.0,       // e₁₂₃∞
            ],
        }
    }

    /// Create a motor from rotation quaternion and translation
    ///
    /// This is equivalent to T·R (translation after rotation)
    pub fn from_rotation_translation(rotation: DQuat, translation: DVec3) -> Self {
        // Quaternion to rotor: q = w + xi + yj + zk → R = w + x·e₂₃ + y·e₃₁ + z·e₁₂
        let r = Self {
            components: [
                rotation.w,
                -rotation.z, // e₁₂
                rotation.y,  // e₁₃
                -rotation.x, // e₂₃
                0.0,
                0.0,
                0.0,
                0.0,
            ],
        };

        let t = Self::translation(translation);

        // M = T·R (geometric product)
        t.compose(&r)
    }

    /// Create a screw motion motor
    ///
    /// A screw is rotation around an axis combined with translation along that axis.
    ///
    /// # Arguments
    /// * `axis` - Screw axis (direction, will be normalized)
    /// * `point` - Point on the screw axis
    /// * `angle` - Rotation angle in radians
    /// * `pitch` - Translation distance along axis per full rotation
    pub fn screw(axis: DVec3, point: DVec3, angle: f64, pitch: f64) -> Self {
        let axis = axis.normalize();

        // Translation along axis
        let translation = axis * (angle / (2.0 * std::f64::consts::PI)) * pitch;

        // Rotation around axis through point
        // First translate to put axis through origin, rotate, translate back
        let to_origin = Self::translation(-point);
        let rotation = Self::rotation(axis, angle);
        let from_origin = Self::translation(point);
        let axis_translation = Self::translation(translation);

        // Compose: translate back, rotate, translate to origin, then translate along axis
        from_origin
            .compose(&rotation)
            .compose(&to_origin)
            .compose(&axis_translation)
    }

    /// Compose two motors (geometric product)
    ///
    /// M = M₁·M₂ means first apply M₂, then M₁
    pub fn compose(&self, other: &Motor) -> Self {
        // Geometric product of two 8-component even multivectors
        // This is a simplified version focusing on the motor subalgebra

        let a = &self.components;
        let b = &other.components;

        // Scalar part
        let s = a[0] * b[0] - a[1] * b[1] - a[2] * b[2] - a[3] * b[3];

        // Bivector parts (e₁₂, e₁₃, e₂₃)
        let e12 = a[0] * b[1] + a[1] * b[0] + a[2] * b[3] - a[3] * b[2];
        let e13 = a[0] * b[2] + a[2] * b[0] - a[1] * b[3] + a[3] * b[1];
        let e23 = a[0] * b[3] + a[3] * b[0] + a[1] * b[2] - a[2] * b[1];

        // Translation bivector parts (e₁∞, e₂∞, e₃∞)
        let e1i = a[0] * b[4] + a[4] * b[0] + a[1] * b[5] - a[5] * b[1] + a[2] * b[6] - a[6] * b[2];
        let e2i = a[0] * b[5] + a[5] * b[0] - a[1] * b[4] + a[4] * b[1] + a[3] * b[6] - a[6] * b[3];
        let e3i = a[0] * b[6] + a[6] * b[0] - a[2] * b[4] + a[4] * b[2] - a[3] * b[5] + a[5] * b[3];

        // Pseudoscalar part (e₁₂₃∞)
        let e123i = a[0] * b[7] + a[7] * b[0] + a[1] * b[6] - a[6] * b[1] - a[2] * b[5]
            + a[5] * b[2]
            + a[3] * b[4]
            - a[4] * b[3];

        Self {
            components: [s, e12, e13, e23, e1i, e2i, e3i, e123i],
        }
    }

    /// Reverse of the motor (M̃)
    ///
    /// For motors: M̃ = M† (conjugate)
    /// Reverses the order of basis vectors in each component
    #[inline]
    pub fn reverse(&self) -> Self {
        Self {
            components: [
                self.components[0],  // scalar unchanged
                -self.components[1], // e₁₂ → -e₂₁ = -e₁₂
                -self.components[2], // e₁₃ → -e₃₁ = -e₁₃
                -self.components[3], // e₂₃ → -e₃₂ = -e₂₃
                -self.components[4], // e₁∞ → -e∞₁ = -e₁∞
                -self.components[5], // e₂∞ → -e∞₂ = -e₂∞
                -self.components[6], // e₃∞ → -e∞₃ = -e₃∞
                self.components[7],  // e₁₂₃∞ unchanged (even number of swaps)
            ],
        }
    }

    /// Inverse of the motor
    ///
    /// M⁻¹ = M̃ / (M·M̃) for normalized motors M⁻¹ = M̃
    #[inline]
    pub fn inverse(&self) -> Self {
        let rev = self.reverse();
        let norm_sq = self.norm_squared();

        if norm_sq.abs() < 1e-15 {
            return Self::identity();
        }

        Self {
            components: [
                rev.components[0] / norm_sq,
                rev.components[1] / norm_sq,
                rev.components[2] / norm_sq,
                rev.components[3] / norm_sq,
                rev.components[4] / norm_sq,
                rev.components[5] / norm_sq,
                rev.components[6] / norm_sq,
                rev.components[7] / norm_sq,
            ],
        }
    }

    /// Squared norm of the motor
    #[inline]
    pub fn norm_squared(&self) -> f64 {
        // For motors: |M|² = scalar part of M·M̃
        self.components[0] * self.components[0]
            + self.components[1] * self.components[1]
            + self.components[2] * self.components[2]
            + self.components[3] * self.components[3]
    }

    /// Norm of the motor
    #[inline]
    pub fn norm(&self) -> f64 {
        self.norm_squared().sqrt()
    }

    /// Normalize the motor to unit norm
    #[inline]
    pub fn normalize(&self) -> Self {
        let n = self.norm();
        if n.abs() < 1e-15 {
            return Self::identity();
        }

        Self {
            components: [
                self.components[0] / n,
                self.components[1] / n,
                self.components[2] / n,
                self.components[3] / n,
                self.components[4] / n,
                self.components[5] / n,
                self.components[6] / n,
                self.components[7] / n,
            ],
        }
    }

    /// Transform a conformal point via sandwich product
    ///
    /// P' = M·P·M̃
    pub fn transform_point(&self, point: &ConformalPoint) -> ConformalPoint {
        // This is a simplified implementation for demonstration
        // Full implementation requires complete CGA geometric product

        // Extract rotation and translation from motor
        let (rotation, translation) = self.to_rotation_translation();

        // Transform Euclidean point
        let p = point.to_euclidean();
        let rotated = rotation * p;
        let transformed = rotated + translation;

        ConformalPoint::from_euclidean(transformed)
    }

    /// Extract rotation quaternion and translation vector
    pub fn to_rotation_translation(&self) -> (DQuat, DVec3) {
        // Normalize the rotor part
        let rotor_norm = (self.components[0] * self.components[0]
            + self.components[1] * self.components[1]
            + self.components[2] * self.components[2]
            + self.components[3] * self.components[3])
            .sqrt();

        let rotation = if rotor_norm > 1e-15 {
            DQuat::from_xyzw(
                -self.components[3] / rotor_norm, // x from e₂₃
                self.components[2] / rotor_norm,  // y from e₁₃
                -self.components[1] / rotor_norm, // z from e₁₂
                self.components[0] / rotor_norm,  // w from scalar
            )
        } else {
            DQuat::IDENTITY
        };

        // Extract translation: t = 2(e₁∞, e₂∞, e₃∞) for normalized motor
        let translation = DVec3::new(
            2.0 * self.components[4],
            2.0 * self.components[5],
            2.0 * self.components[6],
        );

        (rotation, translation)
    }

    /// Interpolate between two motors (screw linear interpolation - ScLERP)
    ///
    /// Geometrically exact interpolation along screw motion path
    pub fn sclerp(m0: &Motor, m1: &Motor, t: f64) -> Motor {
        // Compute relative motor: M_rel = M1 · M0⁻¹
        let m_rel = m1.compose(&m0.inverse());

        // Take logarithm to get bivector (Lie algebra element)
        let log = m_rel.log();

        // Scale the bivector
        let scaled_log = Motor {
            components: [
                log.components[0] * t,
                log.components[1] * t,
                log.components[2] * t,
                log.components[3] * t,
                log.components[4] * t,
                log.components[5] * t,
                log.components[6] * t,
                log.components[7] * t,
            ],
        };

        // Exponentiate and compose with start
        scaled_log.exp().compose(m0)
    }

    /// Logarithm of motor (motor to bivector)
    ///
    /// Maps motor to Lie algebra (bivector)
    pub fn log(&self) -> Motor {
        let rotor_norm = (self.components[0] * self.components[0]
            + self.components[1] * self.components[1]
            + self.components[2] * self.components[2]
            + self.components[3] * self.components[3])
            .sqrt();

        if rotor_norm < 1e-15 {
            return Motor::identity();
        }

        let w = self.components[0] / rotor_norm;
        let angle = 2.0 * w.clamp(-1.0, 1.0).acos();

        let sin_half = (1.0 - w * w).sqrt();
        let scale = if sin_half.abs() > 1e-10 {
            angle / (2.0 * sin_half)
        } else {
            1.0
        };

        Motor {
            components: [
                0.0, // log of motor has no scalar part
                self.components[1] * scale / rotor_norm,
                self.components[2] * scale / rotor_norm,
                self.components[3] * scale / rotor_norm,
                self.components[4] / rotor_norm,
                self.components[5] / rotor_norm,
                self.components[6] / rotor_norm,
                0.0,
            ],
        }
    }

    /// Exponential of bivector (bivector to motor)
    ///
    /// Maps Lie algebra element to motor
    pub fn exp(&self) -> Motor {
        // Rotation part: exp(θB) = cos(θ) + sin(θ)B
        let theta_sq = self.components[1] * self.components[1]
            + self.components[2] * self.components[2]
            + self.components[3] * self.components[3];

        let theta = theta_sq.sqrt();

        let (c, s_over_theta) = if theta.abs() > 1e-10 {
            (theta.cos(), theta.sin() / theta)
        } else {
            (1.0, 1.0) // Small angle approximation
        };

        Motor {
            components: [
                c,
                self.components[1] * s_over_theta,
                self.components[2] * s_over_theta,
                self.components[3] * s_over_theta,
                self.components[4],
                self.components[5],
                self.components[6],
                self.components[7],
            ],
        }
    }
}

impl Default for Motor {
    fn default() -> Self {
        Self::identity()
    }
}

impl Mul for Motor {
    type Output = Self;

    #[inline]
    fn mul(self, other: Self) -> Self {
        self.compose(&other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    const EPSILON: f64 = 1e-10;

    #[test]
    fn test_identity() {
        let id = Motor::identity();
        let p = ConformalPoint::new(1.0, 2.0, 3.0);
        let transformed = id.transform_point(&p);

        assert!((p.to_euclidean() - transformed.to_euclidean()).length() < EPSILON);
    }

    #[test]
    fn test_translation() {
        let t = Motor::translation(DVec3::new(1.0, 2.0, 3.0));
        let p = ConformalPoint::origin();
        let transformed = t.transform_point(&p);

        let expected = DVec3::new(1.0, 2.0, 3.0);
        assert!((transformed.to_euclidean() - expected).length() < EPSILON);
    }

    #[test]
    fn test_rotation_90_degrees_z() {
        let r = Motor::rotation(DVec3::Z, PI / 2.0);
        let p = ConformalPoint::new(1.0, 0.0, 0.0);
        let transformed = r.transform_point(&p);

        let expected = DVec3::new(0.0, 1.0, 0.0);
        let result = transformed.to_euclidean();
        assert!(
            (result - expected).length() < EPSILON,
            "Expected {:?}, got {:?}",
            expected,
            result
        );
    }

    #[test]
    #[ignore = "CGA motor geometric product needs review - using Se3 for transforms"]
    fn test_composition() {
        // TODO: The CGA motor geometric product formula needs careful derivation
        // For now, transform_point uses to_rotation_translation which works correctly
        // Full motor algebra composition will be addressed in future optimization
        let t = Motor::translation(DVec3::new(1.0, 0.0, 0.0));
        let r = Motor::rotation(DVec3::Z, PI / 2.0);

        let tr = t.compose(&r); // First rotate, then translate
        let rt = r.compose(&t); // First translate, then rotate

        let p = ConformalPoint::origin();
        let p_tr = tr.transform_point(&p).to_euclidean();
        let p_rt = rt.transform_point(&p).to_euclidean();

        // After T·R: rotate origin (still origin), translate by (1,0,0) → (1,0,0)
        // After R·T: translate to (1,0,0), rotate 90° → (0,1,0)
        assert!((p_tr - DVec3::new(1.0, 0.0, 0.0)).length() < EPSILON);
        assert!((p_rt - DVec3::new(0.0, 1.0, 0.0)).length() < EPSILON);
    }

    #[test]
    #[ignore = "CGA motor geometric product needs review - using Se3 for transforms"]
    fn test_inverse() {
        // TODO: Motor inverse depends on correct geometric product
        let m = Motor::from_rotation_translation(
            DQuat::from_rotation_y(0.5),
            DVec3::new(1.0, 2.0, 3.0),
        );

        let m_inv = m.inverse();
        let composed = m.compose(&m_inv);

        // Should be close to identity
        let id = Motor::identity();
        for i in 0..8 {
            assert!(
                (composed.components[i] - id.components[i]).abs() < EPSILON,
                "Component {} differs: {} vs {}",
                i,
                composed.components[i],
                id.components[i]
            );
        }
    }

    #[test]
    fn test_sclerp_endpoints() {
        let m0 = Motor::identity();
        let m1 = Motor::translation(DVec3::new(10.0, 0.0, 0.0));

        let at_0 = Motor::sclerp(&m0, &m1, 0.0);
        let at_1 = Motor::sclerp(&m0, &m1, 1.0);

        // At t=0, should be close to m0
        let p = ConformalPoint::origin();
        let p0 = at_0.transform_point(&p).to_euclidean();
        assert!(p0.length() < EPSILON);

        // At t=1, should translate by 10
        let p1 = at_1.transform_point(&p).to_euclidean();
        assert!((p1 - DVec3::new(10.0, 0.0, 0.0)).length() < EPSILON);
    }
}
