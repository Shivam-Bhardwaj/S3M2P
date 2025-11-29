// 2x2 Matrix for 2D transformations and filters
// Extracted from robotics_lib

use glam::Vec2;

/// Simple 2x2 Matrix for 2D operations (EKF, transforms)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mat2 {
    pub m11: f32,
    pub m12: f32,
    pub m21: f32,
    pub m22: f32,
}

impl Mat2 {
    pub const IDENTITY: Self = Self {
        m11: 1.0,
        m12: 0.0,
        m21: 0.0,
        m22: 1.0,
    };

    pub const ZERO: Self = Self {
        m11: 0.0,
        m12: 0.0,
        m21: 0.0,
        m22: 0.0,
    };

    #[inline]
    pub fn new(m11: f32, m12: f32, m21: f32, m22: f32) -> Self {
        Self { m11, m12, m21, m22 }
    }

    /// Create rotation matrix from angle (radians)
    #[inline]
    pub fn from_rotation(angle: f32) -> Self {
        let (sin, cos) = (angle.sin(), angle.cos());
        Self {
            m11: cos,
            m12: -sin,
            m21: sin,
            m22: cos,
        }
    }

    /// Create scale matrix
    #[inline]
    pub fn from_scale(sx: f32, sy: f32) -> Self {
        Self {
            m11: sx,
            m12: 0.0,
            m21: 0.0,
            m22: sy,
        }
    }

    /// Create diagonal matrix
    #[inline]
    pub fn diagonal(d1: f32, d2: f32) -> Self {
        Self::new(d1, 0.0, 0.0, d2)
    }

    /// Multiply matrix by vector
    #[inline]
    pub fn mul_vec(&self, v: Vec2) -> Vec2 {
        Vec2::new(
            self.m11 * v.x + self.m12 * v.y,
            self.m21 * v.x + self.m22 * v.y,
        )
    }

    /// Matrix multiplication
    #[inline]
    pub fn mul(&self, other: Mat2) -> Mat2 {
        Mat2 {
            m11: self.m11 * other.m11 + self.m12 * other.m21,
            m12: self.m11 * other.m12 + self.m12 * other.m22,
            m21: self.m21 * other.m11 + self.m22 * other.m21,
            m22: self.m21 * other.m12 + self.m22 * other.m22,
        }
    }

    /// Transpose
    #[inline]
    pub fn transpose(&self) -> Mat2 {
        Mat2 {
            m11: self.m11,
            m12: self.m21,
            m21: self.m12,
            m22: self.m22,
        }
    }

    /// Matrix addition
    #[inline]
    pub fn add(&self, other: Mat2) -> Mat2 {
        Mat2 {
            m11: self.m11 + other.m11,
            m12: self.m12 + other.m12,
            m21: self.m21 + other.m21,
            m22: self.m22 + other.m22,
        }
    }

    /// Matrix subtraction
    #[inline]
    pub fn sub(&self, other: Mat2) -> Mat2 {
        Mat2 {
            m11: self.m11 - other.m11,
            m12: self.m12 - other.m12,
            m21: self.m21 - other.m21,
            m22: self.m22 - other.m22,
        }
    }

    /// Scalar multiplication
    #[inline]
    pub fn scale(&self, s: f32) -> Mat2 {
        Mat2 {
            m11: self.m11 * s,
            m12: self.m12 * s,
            m21: self.m21 * s,
            m22: self.m22 * s,
        }
    }

    /// Determinant
    #[inline]
    pub fn determinant(&self) -> f32 {
        self.m11 * self.m22 - self.m12 * self.m21
    }

    /// Inverse (returns None if singular)
    #[inline]
    pub fn inverse(&self) -> Option<Mat2> {
        let det = self.determinant();
        if det.abs() < 1e-10 {
            return None;
        }
        let inv_det = 1.0 / det;
        Some(Mat2 {
            m11: self.m22 * inv_det,
            m12: -self.m12 * inv_det,
            m21: -self.m21 * inv_det,
            m22: self.m11 * inv_det,
        })
    }

    /// Trace (sum of diagonal elements)
    #[inline]
    pub fn trace(&self) -> f32 {
        self.m11 + self.m22
    }
}

impl Default for Mat2 {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl std::ops::Mul for Mat2 {
    type Output = Mat2;
    fn mul(self, rhs: Mat2) -> Mat2 {
        Mat2::mul(&self, rhs)
    }
}

impl std::ops::Mul<Vec2> for Mat2 {
    type Output = Vec2;
    fn mul(self, rhs: Vec2) -> Vec2 {
        Mat2::mul_vec(&self, rhs)
    }
}

impl std::ops::Add for Mat2 {
    type Output = Mat2;
    fn add(self, rhs: Mat2) -> Mat2 {
        Mat2::add(&self, rhs)
    }
}

impl std::ops::Sub for Mat2 {
    type Output = Mat2;
    fn sub(self, rhs: Mat2) -> Mat2 {
        Mat2::sub(&self, rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let m = Mat2::IDENTITY;
        let v = Vec2::new(3.0, 4.0);
        assert_eq!(m.mul_vec(v), v);
    }

    #[test]
    fn test_rotation() {
        let m = Mat2::from_rotation(std::f32::consts::FRAC_PI_2); // 90 degrees
        let v = Vec2::new(1.0, 0.0);
        let result = m.mul_vec(v);
        assert!((result.x - 0.0).abs() < 1e-6);
        assert!((result.y - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_inverse() {
        let m = Mat2::new(2.0, 1.0, 1.0, 1.0);
        let inv = m.inverse().unwrap();
        let product = m.mul(inv);
        assert!((product.m11 - 1.0).abs() < 1e-6);
        assert!((product.m22 - 1.0).abs() < 1e-6);
        assert!(product.m12.abs() < 1e-6);
        assert!(product.m21.abs() < 1e-6);
    }

    #[test]
    fn test_singular_no_inverse() {
        let m = Mat2::new(1.0, 2.0, 2.0, 4.0); // Singular
        assert!(m.inverse().is_none());
    }

    #[test]
    fn test_determinant() {
        let m = Mat2::new(3.0, 1.0, 2.0, 4.0);
        assert!((m.determinant() - 10.0).abs() < 1e-6);
    }
}
