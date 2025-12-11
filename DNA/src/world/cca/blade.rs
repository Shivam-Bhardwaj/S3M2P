//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: blade.rs | DNA/src/world/cca/blade.rs
//! PURPOSE: Geometric primitives (blades) in Conformal Geometric Algebra
//! CREATED: 2025-12-09
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! # Blades in CGA
//!
//! In Conformal Geometric Algebra, geometric objects are represented as blades
//! (homogeneous multivectors). Key representations:
//!
//! - **Point**: 1-vector (null vector)
//! - **Point Pair**: 2-blade (two points)
//! - **Circle/Line**: 3-blade (circle = 3 points, line = 2 points + infinity)
//! - **Sphere/Plane**: 4-blade (sphere = 4 points, plane = 3 points + infinity)
//!
//! ## Dual Representations
//!
//! Objects can also be represented in dual form:
//! - Sphere: S* = center - ½r²e∞ (direct) or S = n + de∞ (dual)
//! - Plane: π* = n·e∞ (direct) or π = n + de∞ (dual)
//!
//! ═══════════════════════════════════════════════════════════════════════════════

use super::point::ConformalPoint;
use glam::DVec3;

/// Sphere in Conformal Geometric Algebra
///
/// A sphere can be constructed from:
/// - Center point and radius
/// - Four points on the sphere
/// - Dual representation: S* = C - ½r²e∞
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sphere {
    /// Center of the sphere
    pub center: DVec3,
    /// Radius (positive for real sphere, negative for imaginary)
    pub radius: f64,
}

impl Sphere {
    /// Create a sphere from center and radius
    #[inline]
    pub fn new(center: DVec3, radius: f64) -> Self {
        Self { center, radius }
    }

    /// Create from center coordinates and radius
    #[inline]
    pub fn from_coords(x: f64, y: f64, z: f64, radius: f64) -> Self {
        Self::new(DVec3::new(x, y, z), radius)
    }

    /// Unit sphere centered at origin
    #[inline]
    pub fn unit() -> Self {
        Self::new(DVec3::ZERO, 1.0)
    }

    /// Create from conformal dual representation
    ///
    /// S* = C - ½r²e∞ where C is the conformal center point
    pub fn from_dual(dual: &ConformalPoint) -> Self {
        let center = dual.to_euclidean();
        let c_sq = center.length_squared();

        // For sphere dual S*, the e₊ and e₋ coefficients are:
        // e₊ = ½(|c|² - 1) - ½r²
        // e₋ = ½(|c|² + 1) - ½r²
        // So e₊ + e₋ = |c|² - r², hence r² = |c|² - (e₊ + e₋)
        let e_sum = dual.coords[3] + dual.coords[4];
        let r_sq = c_sq - e_sum;
        let radius = if r_sq >= 0.0 {
            r_sq.sqrt()
        } else {
            -(-r_sq).sqrt()
        };

        Self { center, radius }
    }

    /// Convert to conformal dual representation
    pub fn to_dual(&self) -> ConformalPoint {
        let c = self.center;
        let c_sq = c.length_squared();
        let r_sq = self.radius * self.radius.abs(); // Handle imaginary spheres

        // Sphere dual S* = C - ½r²e∞ where C is the conformal center
        // In [e₁, e₂, e₃, e₊, e₋] basis, center point C has:
        // e₊ = ½(|c|² - 1), e₋ = ½(|c|² + 1)
        // Subtracting ½r²e∞ = ½r²(e₊ + e₋) shifts both components by -½r²
        ConformalPoint {
            coords: [
                c.x,
                c.y,
                c.z,
                0.5 * (c_sq - 1.0) - 0.5 * r_sq, // e₊
                0.5 * (c_sq + 1.0) - 0.5 * r_sq, // e₋
            ],
        }
    }

    /// Check if a point lies on the sphere
    #[inline]
    pub fn contains(&self, point: DVec3) -> bool {
        let dist = (point - self.center).length();
        (dist - self.radius.abs()).abs() < 1e-10
    }

    /// Check if a point is inside the sphere
    #[inline]
    pub fn contains_inside(&self, point: DVec3) -> bool {
        let dist_sq = (point - self.center).length_squared();
        dist_sq < self.radius * self.radius
    }

    /// Distance from point to sphere surface (signed)
    #[inline]
    pub fn distance(&self, point: DVec3) -> f64 {
        (point - self.center).length() - self.radius.abs()
    }

    /// Closest point on sphere to a given point
    pub fn closest_point(&self, point: DVec3) -> DVec3 {
        let dir = point - self.center;
        let dist = dir.length();
        if dist < 1e-15 {
            // Point at center, return arbitrary point on sphere
            self.center + DVec3::X * self.radius.abs()
        } else {
            self.center + dir * (self.radius.abs() / dist)
        }
    }

    /// Surface area
    #[inline]
    pub fn area(&self) -> f64 {
        4.0 * std::f64::consts::PI * self.radius * self.radius
    }

    /// Volume
    #[inline]
    pub fn volume(&self) -> f64 {
        (4.0 / 3.0) * std::f64::consts::PI * self.radius.powi(3)
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Self::unit()
    }
}

/// Plane in Conformal Geometric Algebra
///
/// A plane can be represented in dual form: π = n + de∞
/// where n is the unit normal and d is the distance from origin
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane {
    /// Unit normal vector
    pub normal: DVec3,
    /// Distance from origin (signed)
    pub distance: f64,
}

impl Plane {
    /// Create a plane from normal and distance
    #[inline]
    pub fn new(normal: DVec3, distance: f64) -> Self {
        Self {
            normal: normal.normalize(),
            distance,
        }
    }

    /// Create plane from normal and a point on the plane
    pub fn from_point_normal(point: DVec3, normal: DVec3) -> Self {
        let n = normal.normalize();
        let d = -point.dot(n);
        Self {
            normal: n,
            distance: d,
        }
    }

    /// Create plane from three points
    pub fn from_points(p1: DVec3, p2: DVec3, p3: DVec3) -> Option<Self> {
        let v1 = p2 - p1;
        let v2 = p3 - p1;
        let normal = v1.cross(v2);

        if normal.length() < 1e-15 {
            return None; // Degenerate (collinear points)
        }

        let n = normal.normalize();
        let d = -p1.dot(n);
        Some(Self {
            normal: n,
            distance: d,
        })
    }

    /// XY plane (z = 0)
    #[inline]
    pub fn xy() -> Self {
        Self::new(DVec3::Z, 0.0)
    }

    /// XZ plane (y = 0)
    #[inline]
    pub fn xz() -> Self {
        Self::new(DVec3::Y, 0.0)
    }

    /// YZ plane (x = 0)
    #[inline]
    pub fn yz() -> Self {
        Self::new(DVec3::X, 0.0)
    }

    /// Signed distance from point to plane
    #[inline]
    pub fn signed_distance(&self, point: DVec3) -> f64 {
        point.dot(self.normal) + self.distance
    }

    /// Absolute distance from point to plane
    #[inline]
    pub fn distance_to(&self, point: DVec3) -> f64 {
        self.signed_distance(point).abs()
    }

    /// Project a point onto the plane
    #[inline]
    pub fn project(&self, point: DVec3) -> DVec3 {
        point - self.normal * self.signed_distance(point)
    }

    /// Check if a point lies on the plane
    #[inline]
    pub fn contains(&self, point: DVec3) -> bool {
        self.distance_to(point) < 1e-10
    }

    /// Get a point on the plane (closest to origin)
    #[inline]
    pub fn point_on_plane(&self) -> DVec3 {
        -self.normal * self.distance
    }
}

impl Default for Plane {
    fn default() -> Self {
        Self::xy()
    }
}

/// Circle in Conformal Geometric Algebra
///
/// A circle is the intersection of two spheres, represented as a 3-blade
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Circle {
    /// Center of the circle
    pub center: DVec3,
    /// Normal to the plane containing the circle
    pub normal: DVec3,
    /// Radius of the circle
    pub radius: f64,
}

impl Circle {
    /// Create a circle from center, normal, and radius
    #[inline]
    pub fn new(center: DVec3, normal: DVec3, radius: f64) -> Self {
        Self {
            center,
            normal: normal.normalize(),
            radius,
        }
    }

    /// Unit circle in XY plane
    #[inline]
    pub fn unit_xy() -> Self {
        Self::new(DVec3::ZERO, DVec3::Z, 1.0)
    }

    /// Create circle from three points
    pub fn from_points(p1: DVec3, p2: DVec3, p3: DVec3) -> Option<Self> {
        // Find the plane containing the three points
        let v1 = p2 - p1;
        let v2 = p3 - p1;
        let normal = v1.cross(v2);

        if normal.length() < 1e-15 {
            return None; // Collinear points
        }

        let n = normal.normalize();

        // Find circumcenter in the plane
        let d1 = p1.length_squared();
        let d2 = p2.length_squared();
        let d3 = p3.length_squared();

        let a = 2.0 * (p1.x * (p2.y - p3.y) + p2.x * (p3.y - p1.y) + p3.x * (p1.y - p2.y));

        if a.abs() < 1e-15 {
            return None;
        }

        let cx = (d1 * (p2.y - p3.y) + d2 * (p3.y - p1.y) + d3 * (p1.y - p2.y)) / a;
        let cy = (d1 * (p3.x - p2.x) + d2 * (p1.x - p3.x) + d3 * (p2.x - p1.x)) / a;

        // Get plane reference point
        let plane_origin = (p1 + p2 + p3) / 3.0;

        // Build local coordinate system
        let u = v1.normalize();
        let v = n.cross(u).normalize();

        let center = plane_origin + u * (cx - plane_origin.dot(u)) + v * (cy - plane_origin.dot(v));
        let radius = (p1 - center).length();

        Some(Self {
            center,
            normal: n,
            radius,
        })
    }

    /// Get a point on the circle at angle theta
    ///
    /// For unit_xy (normal = Z), theta=0 gives X, theta=π/2 gives Y
    pub fn point_at(&self, theta: f64) -> DVec3 {
        // Build orthonormal basis in the plane of the circle
        // Choose a reference vector that's not parallel to normal
        let ref_vec = if self.normal.z.abs() > 0.9 {
            DVec3::X // Normal is close to Z, use X as reference
        } else if self.normal.y.abs() > 0.9 {
            DVec3::Z // Normal is close to Y, use Z as reference
        } else {
            DVec3::Y // Normal is close to X, use Y as reference
        };

        // Project reference onto the plane (perpendicular to normal)
        let u = (ref_vec - self.normal * ref_vec.dot(self.normal)).normalize();
        // v completes right-handed basis: v = normal × u
        let v = self.normal.cross(u);

        self.center + self.radius * (u * theta.cos() + v * theta.sin())
    }

    /// Circumference
    #[inline]
    pub fn circumference(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }

    /// Area enclosed by circle
    #[inline]
    pub fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }

    /// Plane containing the circle
    #[inline]
    pub fn plane(&self) -> Plane {
        Plane::from_point_normal(self.center, self.normal)
    }
}

impl Default for Circle {
    fn default() -> Self {
        Self::unit_xy()
    }
}

/// Line in Conformal Geometric Algebra
///
/// A line is represented as a circle with infinite radius (passing through e∞)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line {
    /// A point on the line
    pub point: DVec3,
    /// Direction of the line (unit vector)
    pub direction: DVec3,
}

impl Line {
    /// Create a line from point and direction
    #[inline]
    pub fn new(point: DVec3, direction: DVec3) -> Self {
        Self {
            point,
            direction: direction.normalize(),
        }
    }

    /// Create from two points
    pub fn from_points(p1: DVec3, p2: DVec3) -> Option<Self> {
        let dir = p2 - p1;
        if dir.length() < 1e-15 {
            return None;
        }
        Some(Self::new(p1, dir))
    }

    /// X axis
    #[inline]
    pub fn x_axis() -> Self {
        Self::new(DVec3::ZERO, DVec3::X)
    }

    /// Y axis
    #[inline]
    pub fn y_axis() -> Self {
        Self::new(DVec3::ZERO, DVec3::Y)
    }

    /// Z axis
    #[inline]
    pub fn z_axis() -> Self {
        Self::new(DVec3::ZERO, DVec3::Z)
    }

    /// Get point on line at parameter t: p + t*d
    #[inline]
    pub fn point_at(&self, t: f64) -> DVec3 {
        self.point + self.direction * t
    }

    /// Closest point on line to a given point
    pub fn closest_point(&self, point: DVec3) -> DVec3 {
        let v = point - self.point;
        let t = v.dot(self.direction);
        self.point + self.direction * t
    }

    /// Distance from point to line
    #[inline]
    pub fn distance(&self, point: DVec3) -> f64 {
        let closest = self.closest_point(point);
        (point - closest).length()
    }

    /// Project point onto line
    #[inline]
    pub fn project(&self, point: DVec3) -> DVec3 {
        self.closest_point(point)
    }
}

impl Default for Line {
    fn default() -> Self {
        Self::x_axis()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    const EPSILON: f64 = 1e-10;

    #[test]
    fn test_sphere_contains() {
        let sphere = Sphere::new(DVec3::ZERO, 1.0);

        assert!(sphere.contains(DVec3::X));
        assert!(sphere.contains(DVec3::Y));
        assert!(sphere.contains(DVec3::Z));
        assert!(!sphere.contains(DVec3::new(2.0, 0.0, 0.0)));
    }

    #[test]
    fn test_sphere_dual_roundtrip() {
        let sphere = Sphere::new(DVec3::new(1.0, 2.0, 3.0), 5.0);
        let dual = sphere.to_dual();
        let back = Sphere::from_dual(&dual);

        assert!((sphere.center - back.center).length() < EPSILON);
        assert!((sphere.radius - back.radius).abs() < EPSILON);
    }

    #[test]
    fn test_plane_distance() {
        let plane = Plane::xy(); // z = 0
        let point = DVec3::new(1.0, 2.0, 3.0);

        assert!((plane.signed_distance(point) - 3.0).abs() < EPSILON);
    }

    #[test]
    fn test_plane_project() {
        let plane = Plane::xy();
        let point = DVec3::new(1.0, 2.0, 3.0);
        let projected = plane.project(point);

        assert!((projected - DVec3::new(1.0, 2.0, 0.0)).length() < EPSILON);
    }

    #[test]
    fn test_circle_point_at() {
        let circle = Circle::unit_xy();

        let p0 = circle.point_at(0.0);
        assert!((p0 - DVec3::X).length() < EPSILON);

        let p90 = circle.point_at(PI / 2.0);
        assert!((p90 - DVec3::Y).length() < EPSILON);
    }

    #[test]
    fn test_line_distance() {
        let line = Line::x_axis();
        let point = DVec3::new(5.0, 3.0, 4.0);

        let dist = line.distance(point);
        assert!((dist - 5.0).abs() < EPSILON); // sqrt(3² + 4²) = 5
    }

    #[test]
    fn test_line_closest_point() {
        let line = Line::x_axis();
        let point = DVec3::new(5.0, 3.0, 0.0);
        let closest = line.closest_point(point);

        assert!((closest - DVec3::new(5.0, 0.0, 0.0)).length() < EPSILON);
    }
}
