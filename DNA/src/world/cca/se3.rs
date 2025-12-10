//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: se3.rs | DNA/src/world/cca/se3.rs
//! PURPOSE: SE(3) Lie group and se(3) Lie algebra for frame transformations
//! CREATED: 2025-12-09
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! # SE(3) - Special Euclidean Group
//!
//! SE(3) is the Lie group of rigid body transformations in 3D space, consisting
//! of all rotations (SO(3)) and translations.
//!
//! ## Structure
//!
//! An element of SE(3) can be represented as:
//! ```text
//! g = (R, t)  where R ∈ SO(3), t ∈ ℝ³
//!
//! Or as a 4×4 matrix:
//! ┌       ┐
//! │ R   t │
//! │ 0   1 │
//! └       ┘
//! ```
//!
//! ## Lie Algebra se(3)
//!
//! The tangent space at identity (Lie algebra) consists of 6D twist vectors:
//! ```text
//! ξ = (ω, v)  where ω ∈ ℝ³ (angular velocity), v ∈ ℝ³ (linear velocity)
//! ```
//!
//! ## Key Operations
//!
//! - `exp: se(3) → SE(3)` - Exponential map (twist to transformation)
//! - `log: SE(3) → se(3)` - Logarithm map (transformation to twist)
//! - `Ad: SE(3) × se(3) → se(3)` - Adjoint action (transform velocities)
//!
//! ═══════════════════════════════════════════════════════════════════════════════

use glam::{DMat3, DMat4, DQuat, DVec3};
use std::ops::Mul;

/// SO(3) - Special Orthogonal Group (3D rotations)
///
/// Represented internally as a quaternion for numerical stability
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SO3 {
    /// Unit quaternion representing the rotation
    pub quat: DQuat,
}

impl SO3 {
    /// Identity rotation
    #[inline]
    pub fn identity() -> Self {
        Self {
            quat: DQuat::IDENTITY,
        }
    }

    /// Create from quaternion (will be normalized)
    #[inline]
    pub fn from_quat(quat: DQuat) -> Self {
        Self {
            quat: quat.normalize(),
        }
    }

    /// Create from axis-angle representation
    #[inline]
    pub fn from_axis_angle(axis: DVec3, angle: f64) -> Self {
        Self {
            quat: DQuat::from_axis_angle(axis.normalize(), angle),
        }
    }

    /// Create from rotation matrix
    pub fn from_matrix(m: DMat3) -> Self {
        Self {
            quat: DQuat::from_mat3(&m),
        }
    }

    /// Create rotation around X axis
    #[inline]
    pub fn from_rotation_x(angle: f64) -> Self {
        Self {
            quat: DQuat::from_rotation_x(angle),
        }
    }

    /// Create rotation around Y axis
    #[inline]
    pub fn from_rotation_y(angle: f64) -> Self {
        Self {
            quat: DQuat::from_rotation_y(angle),
        }
    }

    /// Create rotation around Z axis
    #[inline]
    pub fn from_rotation_z(angle: f64) -> Self {
        Self {
            quat: DQuat::from_rotation_z(angle),
        }
    }

    /// Convert to rotation matrix
    #[inline]
    pub fn to_matrix(&self) -> DMat3 {
        DMat3::from_quat(self.quat)
    }

    /// Convert to axis-angle representation
    #[inline]
    pub fn to_axis_angle(&self) -> (DVec3, f64) {
        self.quat.to_axis_angle()
    }

    /// Rotate a vector
    #[inline]
    pub fn rotate(&self, v: DVec3) -> DVec3 {
        self.quat * v
    }

    /// Compose rotations: self * other (apply other first, then self)
    #[inline]
    pub fn compose(&self, other: &SO3) -> SO3 {
        SO3 {
            quat: self.quat * other.quat,
        }
    }

    /// Inverse rotation
    #[inline]
    pub fn inverse(&self) -> SO3 {
        SO3 {
            quat: self.quat.conjugate(),
        }
    }

    /// Exponential map: so(3) → SO(3)
    ///
    /// Maps angular velocity vector to rotation
    pub fn exp(omega: DVec3) -> SO3 {
        let theta = omega.length();
        if theta < 1e-10 {
            // Small angle approximation
            SO3 {
                quat: DQuat::from_xyzw(omega.x / 2.0, omega.y / 2.0, omega.z / 2.0, 1.0)
                    .normalize(),
            }
        } else {
            let axis = omega / theta;
            SO3::from_axis_angle(axis, theta)
        }
    }

    /// Logarithm map: SO(3) → so(3)
    ///
    /// Maps rotation to angular velocity vector
    pub fn log(&self) -> DVec3 {
        let (axis, angle) = self.to_axis_angle();
        axis * angle
    }

    /// Spherical linear interpolation
    #[inline]
    pub fn slerp(&self, other: &SO3, t: f64) -> SO3 {
        SO3 {
            quat: self.quat.slerp(other.quat, t),
        }
    }
}

impl Default for SO3 {
    fn default() -> Self {
        Self::identity()
    }
}

impl Mul for SO3 {
    type Output = Self;

    #[inline]
    fn mul(self, other: Self) -> Self {
        self.compose(&other)
    }
}

impl Mul<DVec3> for SO3 {
    type Output = DVec3;

    #[inline]
    fn mul(self, v: DVec3) -> DVec3 {
        self.rotate(v)
    }
}

/// SE(3) - Special Euclidean Group (rigid transformations)
///
/// Represents rotation and translation as a single transformation
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Se3 {
    /// Rotation component
    pub rotation: SO3,
    /// Translation component
    pub translation: DVec3,
}

impl Se3 {
    /// Identity transformation
    #[inline]
    pub fn identity() -> Self {
        Self {
            rotation: SO3::identity(),
            translation: DVec3::ZERO,
        }
    }

    /// Create from rotation and translation
    #[inline]
    pub fn new(rotation: SO3, translation: DVec3) -> Self {
        Self {
            rotation,
            translation,
        }
    }

    /// Create pure translation
    #[inline]
    pub fn from_translation(translation: DVec3) -> Self {
        Self {
            rotation: SO3::identity(),
            translation,
        }
    }

    /// Create pure rotation
    #[inline]
    pub fn from_rotation(rotation: SO3) -> Self {
        Self {
            rotation,
            translation: DVec3::ZERO,
        }
    }

    /// Create rotation around X axis
    #[inline]
    pub fn from_rotation_x(angle: f64) -> Self {
        Self::from_rotation(SO3::from_rotation_x(angle))
    }

    /// Create rotation around Y axis
    #[inline]
    pub fn from_rotation_y(angle: f64) -> Self {
        Self::from_rotation(SO3::from_rotation_y(angle))
    }

    /// Create rotation around Z axis
    #[inline]
    pub fn from_rotation_z(angle: f64) -> Self {
        Self::from_rotation(SO3::from_rotation_z(angle))
    }

    /// Create from 4×4 homogeneous matrix
    pub fn from_matrix(m: DMat4) -> Self {
        let rotation = SO3::from_matrix(DMat3::from_cols(
            DVec3::new(m.col(0).x, m.col(0).y, m.col(0).z),
            DVec3::new(m.col(1).x, m.col(1).y, m.col(1).z),
            DVec3::new(m.col(2).x, m.col(2).y, m.col(2).z),
        ));
        let translation = DVec3::new(m.col(3).x, m.col(3).y, m.col(3).z);

        Self {
            rotation,
            translation,
        }
    }

    /// Convert to 4×4 homogeneous matrix
    pub fn to_matrix(&self) -> DMat4 {
        let r = self.rotation.to_matrix();
        let t = self.translation;

        DMat4::from_cols(
            glam::DVec4::new(r.col(0).x, r.col(0).y, r.col(0).z, 0.0),
            glam::DVec4::new(r.col(1).x, r.col(1).y, r.col(1).z, 0.0),
            glam::DVec4::new(r.col(2).x, r.col(2).y, r.col(2).z, 0.0),
            glam::DVec4::new(t.x, t.y, t.z, 1.0),
        )
    }

    /// Transform a point: p' = R·p + t
    #[inline]
    pub fn transform_point(&self, p: DVec3) -> DVec3 {
        self.rotation.rotate(p) + self.translation
    }

    /// Transform a vector (rotation only): v' = R·v
    #[inline]
    pub fn transform_vector(&self, v: DVec3) -> DVec3 {
        self.rotation.rotate(v)
    }

    /// Compose transformations: g₁ * g₂ = (R₁R₂, R₁t₂ + t₁)
    #[inline]
    pub fn compose(&self, other: &Se3) -> Se3 {
        Se3 {
            rotation: self.rotation.compose(&other.rotation),
            translation: self.rotation.rotate(other.translation) + self.translation,
        }
    }

    /// Inverse transformation: g⁻¹ = (R⁻¹, -R⁻¹t)
    #[inline]
    pub fn inverse(&self) -> Se3 {
        let r_inv = self.rotation.inverse();
        Se3 {
            rotation: r_inv,
            translation: r_inv.rotate(-self.translation),
        }
    }

    /// Exponential map: se(3) → SE(3)
    ///
    /// Maps a twist vector (ω, v) to a transformation
    pub fn exp(twist: &se3) -> Se3 {
        let theta = twist.omega.length();

        if theta < 1e-10 {
            // Pure translation (or small rotation)
            Se3 {
                rotation: SO3::exp(twist.omega),
                translation: twist.v,
            }
        } else {
            // General case using Rodrigues formula
            let axis = twist.omega / theta;
            let rotation = SO3::from_axis_angle(axis, theta);

            // V matrix for translation
            let omega_hat = skew(twist.omega);
            let v_matrix = DMat3::IDENTITY
                + omega_hat * ((1.0 - theta.cos()) / (theta * theta))
                + (omega_hat * omega_hat) * ((theta - theta.sin()) / (theta * theta * theta));

            let translation = v_matrix * twist.v;

            Se3 {
                rotation,
                translation,
            }
        }
    }

    /// Logarithm map: SE(3) → se(3)
    ///
    /// Maps a transformation to a twist vector
    pub fn log(&self) -> se3 {
        let omega = self.rotation.log();
        let theta = omega.length();

        if theta < 1e-10 {
            // Near identity rotation
            se3 {
                omega,
                v: self.translation,
            }
        } else {
            // General case
            let _axis = omega / theta;
            let omega_hat = skew(omega);

            // V⁻¹ matrix
            let half_theta = theta / 2.0;
            let v_inv = DMat3::IDENTITY - omega_hat * 0.5
                + omega_hat
                    * omega_hat
                    * (1.0 / (theta * theta))
                    * (1.0 - half_theta / half_theta.tan());

            let v = v_inv * self.translation;

            se3 { omega, v }
        }
    }

    /// Adjoint matrix (6×6)
    ///
    /// Used for covariance propagation in Lie algebra
    pub fn adjoint_matrix(&self) -> [[f64; 6]; 6] {
        let r = self.rotation.to_matrix();
        let t_hat = skew(self.translation);
        let t_hat_r = t_hat * r;

        let mut ad = [[0.0; 6]; 6];

        // Top-left 3×3: R
        for (j, col) in [r.col(0), r.col(1), r.col(2)].iter().enumerate() {
            for (i, &val) in col.to_array().iter().enumerate() {
                ad[i][j] = val;
            }
        }

        // Top-right 3×3: 0
        // (already initialized to 0)

        // Bottom-left 3×3: [t]×R
        for (j, col) in [t_hat_r.col(0), t_hat_r.col(1), t_hat_r.col(2)]
            .iter()
            .enumerate()
        {
            for (i, &val) in col.to_array().iter().enumerate() {
                ad[i + 3][j] = val;
            }
        }

        // Bottom-right 3×3: R
        for (j, col) in [r.col(0), r.col(1), r.col(2)].iter().enumerate() {
            for (i, &val) in col.to_array().iter().enumerate() {
                ad[i + 3][j + 3] = val;
            }
        }

        ad
    }

    /// Apply adjoint action to a twist: Ad_g(ξ)
    #[inline]
    pub fn adjoint(&self, twist: &se3) -> se3 {
        se3 {
            omega: self.rotation.rotate(twist.omega),
            v: self.rotation.rotate(twist.v)
                + self.translation.cross(self.rotation.rotate(twist.omega)),
        }
    }

    /// Interpolate between transformations
    pub fn interpolate(&self, other: &Se3, t: f64) -> Se3 {
        // Compute relative transformation
        let rel = self.inverse().compose(other);

        // Take log, scale, and exp
        let rel_twist = rel.log();
        let scaled_twist = se3 {
            omega: rel_twist.omega * t,
            v: rel_twist.v * t,
        };

        self.compose(&Se3::exp(&scaled_twist))
    }
}

impl Default for Se3 {
    fn default() -> Self {
        Self::identity()
    }
}

impl Mul for Se3 {
    type Output = Self;

    #[inline]
    fn mul(self, other: Self) -> Self {
        self.compose(&other)
    }
}

/// se(3) - Lie algebra of SE(3)
///
/// A 6D twist vector representing infinitesimal rigid motion
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct se3 {
    /// Angular velocity (rotation rate around each axis)
    pub omega: DVec3,
    /// Linear velocity
    pub v: DVec3,
}

impl se3 {
    /// Zero twist
    #[inline]
    pub fn zero() -> Self {
        Self {
            omega: DVec3::ZERO,
            v: DVec3::ZERO,
        }
    }

    /// Create from angular and linear velocity
    #[inline]
    pub fn new(omega: DVec3, v: DVec3) -> Self {
        Self { omega, v }
    }

    /// Create from 6-vector [ω, v]
    #[inline]
    pub fn from_vector(vec: [f64; 6]) -> Self {
        Self {
            omega: DVec3::new(vec[0], vec[1], vec[2]),
            v: DVec3::new(vec[3], vec[4], vec[5]),
        }
    }

    /// Convert to 6-vector
    #[inline]
    pub fn to_vector(&self) -> [f64; 6] {
        [
            self.omega.x,
            self.omega.y,
            self.omega.z,
            self.v.x,
            self.v.y,
            self.v.z,
        ]
    }

    /// Squared norm of the twist
    #[inline]
    pub fn norm_squared(&self) -> f64 {
        self.omega.length_squared() + self.v.length_squared()
    }

    /// Norm of the twist
    #[inline]
    pub fn norm(&self) -> f64 {
        self.norm_squared().sqrt()
    }

    /// Scale the twist
    #[inline]
    pub fn scale(&self, s: f64) -> Self {
        Self {
            omega: self.omega * s,
            v: self.v * s,
        }
    }

    /// Add two twists
    #[inline]
    pub fn add(&self, other: &Self) -> Self {
        Self {
            omega: self.omega + other.omega,
            v: self.v + other.v,
        }
    }

    /// Subtract two twists
    #[inline]
    pub fn sub(&self, other: &Self) -> Self {
        Self {
            omega: self.omega - other.omega,
            v: self.v - other.v,
        }
    }

    /// Lie bracket [ξ₁, ξ₂]
    #[inline]
    pub fn bracket(&self, other: &Self) -> Self {
        Self {
            omega: self.omega.cross(other.omega),
            v: self.omega.cross(other.v) + self.v.cross(other.omega),
        }
    }
}

/// Skew-symmetric matrix from vector
///
/// [v]× = | 0   -v₃   v₂ |
///        | v₃   0   -v₁ |
///        |-v₂   v₁   0  |
#[inline]
fn skew(v: DVec3) -> DMat3 {
    DMat3::from_cols(
        DVec3::new(0.0, v.z, -v.y),
        DVec3::new(-v.z, 0.0, v.x),
        DVec3::new(v.y, -v.x, 0.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    const EPSILON: f64 = 1e-10;

    #[test]
    fn test_so3_identity() {
        let id = SO3::identity();
        let v = DVec3::new(1.0, 2.0, 3.0);
        let rotated = id.rotate(v);

        assert!((v - rotated).length() < EPSILON);
    }

    #[test]
    fn test_so3_rotation_z_90() {
        let r = SO3::from_rotation_z(PI / 2.0);
        let v = DVec3::new(1.0, 0.0, 0.0);
        let rotated = r.rotate(v);

        let expected = DVec3::new(0.0, 1.0, 0.0);
        assert!((rotated - expected).length() < EPSILON);
    }

    #[test]
    fn test_so3_exp_log_roundtrip() {
        let omega = DVec3::new(0.1, 0.2, 0.3);
        let r = SO3::exp(omega);
        let omega_back = r.log();

        assert!((omega - omega_back).length() < EPSILON);
    }

    #[test]
    fn test_se3_identity() {
        let id = Se3::identity();
        let p = DVec3::new(1.0, 2.0, 3.0);
        let transformed = id.transform_point(p);

        assert!((p - transformed).length() < EPSILON);
    }

    #[test]
    fn test_se3_translation() {
        let t = Se3::from_translation(DVec3::new(1.0, 2.0, 3.0));
        let p = DVec3::ZERO;
        let transformed = t.transform_point(p);

        let expected = DVec3::new(1.0, 2.0, 3.0);
        assert!((transformed - expected).length() < EPSILON);
    }

    #[test]
    fn test_se3_composition() {
        let t = Se3::from_translation(DVec3::new(1.0, 0.0, 0.0));
        let r = Se3::from_rotation(SO3::from_rotation_z(PI / 2.0));

        // First rotate, then translate
        let rt = r.compose(&t);
        let p = DVec3::ZERO;

        // t(0,0,0) = (1,0,0), then r(1,0,0) = (0,1,0)
        let result = rt.transform_point(p);
        let expected = DVec3::new(0.0, 1.0, 0.0);
        assert!((result - expected).length() < EPSILON);
    }

    #[test]
    fn test_se3_inverse() {
        let g = Se3::new(SO3::from_rotation_y(0.5), DVec3::new(1.0, 2.0, 3.0));
        let g_inv = g.inverse();
        let composed = g.compose(&g_inv);

        // Should be close to identity
        let p = DVec3::new(1.0, 1.0, 1.0);
        let transformed = composed.transform_point(p);
        assert!((p - transformed).length() < EPSILON);
    }

    #[test]
    fn test_se3_exp_log_roundtrip() {
        let twist = se3::new(DVec3::new(0.1, 0.2, 0.3), DVec3::new(1.0, 2.0, 3.0));

        let g = Se3::exp(&twist);
        let twist_back = g.log();

        assert!((twist.omega - twist_back.omega).length() < EPSILON);
        assert!((twist.v - twist_back.v).length() < EPSILON);
    }

    #[test]
    fn test_se3_interpolate() {
        let g0 = Se3::identity();
        let g1 = Se3::from_translation(DVec3::new(10.0, 0.0, 0.0));

        let g_mid = g0.interpolate(&g1, 0.5);
        let p = DVec3::ZERO;
        let result = g_mid.transform_point(p);

        // Should be halfway
        let expected = DVec3::new(5.0, 0.0, 0.0);
        assert!((result - expected).length() < EPSILON);
    }
}
