//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: geometry.rs
//! PATH: DNA/src/cad/geometry.rs
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! PURPOSE: Geometric primitives for CAD/B-Rep modeling
//!
//! LAYER: DNA → CAD
//!
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │ TYPES DEFINED                                                               │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │ Point3        3D point in space                                             │
//! │ Vector3       3D direction vector                                           │
//! │ Plane         Infinite plane (point + normal)                               │
//! │ Line          Infinite line (point + direction)                             │
//! │ Ray           Half-infinite ray (origin + direction)                        │
//! │ Segment       Line segment (two endpoints)                                  │
//! │ BoundingBox3  Axis-aligned bounding box                                     │
//! │ Transform3    Affine transformation matrix                                  │
//! └─────────────────────────────────────────────────────────────────────────────┘
//!
//! DEPENDS ON:
//!   • glam - Vector math
//!
//! USED BY:
//!   • DNA/src/cad/topology.rs → Geometric backing for topology
//!   • CORE/CAD_ENGINE         → Solid modeling
//!
//! ═══════════════════════════════════════════════════════════════════════════════

use glam::{Mat4, Vec3};

/// Tolerance for geometric comparisons (1 micrometer)
pub const TOLERANCE: f32 = 1e-6;

/// 3D point in space
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3 {
    pub const ORIGIN: Self = Self { x: 0.0, y: 0.0, z: 0.0 };

    #[inline]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn from_vec3(v: Vec3) -> Self {
        Self { x: v.x, y: v.y, z: v.z }
    }

    #[inline]
    pub fn to_vec3(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    /// Distance to another point
    #[inline]
    pub fn distance(self, other: Point3) -> f32 {
        (self.to_vec3() - other.to_vec3()).length()
    }

    /// Squared distance (faster, no sqrt)
    #[inline]
    pub fn distance_squared(self, other: Point3) -> f32 {
        (self.to_vec3() - other.to_vec3()).length_squared()
    }

    /// Check if two points are approximately equal
    #[inline]
    pub fn approx_eq(self, other: Point3, tolerance: f32) -> bool {
        self.distance_squared(other) < tolerance * tolerance
    }

    /// Linear interpolation between two points
    #[inline]
    pub fn lerp(self, other: Point3, t: f32) -> Point3 {
        Point3::from_vec3(self.to_vec3().lerp(other.to_vec3(), t))
    }

    /// Midpoint between two points
    #[inline]
    pub fn midpoint(self, other: Point3) -> Point3 {
        self.lerp(other, 0.5)
    }

    /// Transform point by matrix
    #[inline]
    pub fn transform(self, matrix: &Transform3) -> Point3 {
        let v = matrix.0.transform_point3(self.to_vec3());
        Point3::from_vec3(v)
    }
}

impl From<Vec3> for Point3 {
    fn from(v: Vec3) -> Self {
        Self::from_vec3(v)
    }
}

impl From<Point3> for Vec3 {
    fn from(p: Point3) -> Self {
        p.to_vec3()
    }
}

impl std::ops::Add<Vector3> for Point3 {
    type Output = Point3;
    fn add(self, v: Vector3) -> Point3 {
        Point3::new(self.x + v.x, self.y + v.y, self.z + v.z)
    }
}

impl std::ops::Sub for Point3 {
    type Output = Vector3;
    fn sub(self, other: Point3) -> Vector3 {
        Vector3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

/// 3D direction vector
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const X: Self = Self { x: 1.0, y: 0.0, z: 0.0 };
    pub const Y: Self = Self { x: 0.0, y: 1.0, z: 0.0 };
    pub const Z: Self = Self { x: 0.0, y: 0.0, z: 1.0 };
    pub const NEG_X: Self = Self { x: -1.0, y: 0.0, z: 0.0 };
    pub const NEG_Y: Self = Self { x: 0.0, y: -1.0, z: 0.0 };
    pub const NEG_Z: Self = Self { x: 0.0, y: 0.0, z: -1.0 };

    #[inline]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn from_vec3(v: Vec3) -> Self {
        Self { x: v.x, y: v.y, z: v.z }
    }

    #[inline]
    pub fn to_vec3(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }

    #[inline]
    pub fn length(self) -> f32 {
        self.to_vec3().length()
    }

    #[inline]
    pub fn length_squared(self) -> f32 {
        self.to_vec3().length_squared()
    }

    /// Normalize to unit vector (returns None if zero-length)
    pub fn normalize(self) -> Option<Self> {
        let len = self.length();
        if len < TOLERANCE {
            None
        } else {
            Some(Self::new(self.x / len, self.y / len, self.z / len))
        }
    }

    /// Normalize, defaulting to Z if zero-length
    #[inline]
    pub fn normalize_or_z(self) -> Self {
        self.normalize().unwrap_or(Self::Z)
    }

    #[inline]
    pub fn dot(self, other: Vector3) -> f32 {
        self.to_vec3().dot(other.to_vec3())
    }

    #[inline]
    pub fn cross(self, other: Vector3) -> Vector3 {
        Vector3::from_vec3(self.to_vec3().cross(other.to_vec3()))
    }

    /// Angle between two vectors in radians
    pub fn angle(self, other: Vector3) -> f32 {
        let dot = self.dot(other);
        let len_product = self.length() * other.length();
        if len_product < TOLERANCE {
            0.0
        } else {
            (dot / len_product).clamp(-1.0, 1.0).acos()
        }
    }

    /// Project this vector onto another
    pub fn project_onto(self, other: Vector3) -> Vector3 {
        let other_len_sq = other.length_squared();
        if other_len_sq < TOLERANCE {
            Vector3::ZERO
        } else {
            other * (self.dot(other) / other_len_sq)
        }
    }

    /// Transform vector by matrix (direction only, no translation)
    #[inline]
    pub fn transform(self, matrix: &Transform3) -> Vector3 {
        let v = matrix.0.transform_vector3(self.to_vec3());
        Vector3::from_vec3(v)
    }
}

impl From<Vec3> for Vector3 {
    fn from(v: Vec3) -> Self {
        Self::from_vec3(v)
    }
}

impl std::ops::Add for Vector3 {
    type Output = Vector3;
    fn add(self, other: Vector3) -> Vector3 {
        Vector3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl std::ops::Sub for Vector3 {
    type Output = Vector3;
    fn sub(self, other: Vector3) -> Vector3 {
        Vector3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl std::ops::Mul<f32> for Vector3 {
    type Output = Vector3;
    fn mul(self, scalar: f32) -> Vector3 {
        Vector3::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl std::ops::Neg for Vector3 {
    type Output = Vector3;
    fn neg(self) -> Vector3 {
        Vector3::new(-self.x, -self.y, -self.z)
    }
}

/// Infinite plane defined by point and normal
#[derive(Clone, Copy, Debug)]
pub struct Plane {
    pub origin: Point3,
    pub normal: Vector3,
}

impl Plane {
    pub const XY: Self = Self { origin: Point3::ORIGIN, normal: Vector3::Z };
    pub const XZ: Self = Self { origin: Point3::ORIGIN, normal: Vector3::Y };
    pub const YZ: Self = Self { origin: Point3::ORIGIN, normal: Vector3::X };

    pub fn new(origin: Point3, normal: Vector3) -> Option<Self> {
        let normal = normal.normalize()?;
        Some(Self { origin, normal })
    }

    /// Create plane from three non-collinear points
    pub fn from_points(p1: Point3, p2: Point3, p3: Point3) -> Option<Self> {
        let v1 = p2 - p1;
        let v2 = p3 - p1;
        let normal = v1.cross(v2).normalize()?;
        Some(Self { origin: p1, normal })
    }

    /// Signed distance from point to plane (positive = same side as normal)
    pub fn signed_distance(&self, point: Point3) -> f32 {
        (point - self.origin).dot(self.normal)
    }

    /// Project point onto plane
    pub fn project_point(&self, point: Point3) -> Point3 {
        let dist = self.signed_distance(point);
        point + self.normal * (-dist)
    }
}

/// Infinite line defined by point and direction
#[derive(Clone, Copy, Debug)]
pub struct Line {
    pub origin: Point3,
    pub direction: Vector3,
}

impl Line {
    pub fn new(origin: Point3, direction: Vector3) -> Option<Self> {
        let direction = direction.normalize()?;
        Some(Self { origin, direction })
    }

    /// Create line through two points
    pub fn from_points(p1: Point3, p2: Point3) -> Option<Self> {
        Self::new(p1, p2 - p1)
    }

    /// Point at parameter t along line
    #[inline]
    pub fn point_at(&self, t: f32) -> Point3 {
        self.origin + self.direction * t
    }

    /// Project point onto line, returning parameter t
    pub fn project_param(&self, point: Point3) -> f32 {
        (point - self.origin).dot(self.direction)
    }

    /// Project point onto line
    pub fn project_point(&self, point: Point3) -> Point3 {
        self.point_at(self.project_param(point))
    }

    /// Distance from point to line
    pub fn distance_to_point(&self, point: Point3) -> f32 {
        let projected = self.project_point(point);
        point.distance(projected)
    }
}

/// Half-infinite ray (origin + direction)
#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vector3,
}

impl Ray {
    pub fn new(origin: Point3, direction: Vector3) -> Option<Self> {
        let direction = direction.normalize()?;
        Some(Self { origin, direction })
    }

    #[inline]
    pub fn point_at(&self, t: f32) -> Point3 {
        self.origin + self.direction * t
    }

    /// Intersect ray with plane, returning parameter t (None if parallel)
    pub fn intersect_plane(&self, plane: &Plane) -> Option<f32> {
        let denom = self.direction.dot(plane.normal);
        if denom.abs() < TOLERANCE {
            return None; // Parallel
        }
        let t = (plane.origin - self.origin).dot(plane.normal) / denom;
        if t >= 0.0 { Some(t) } else { None }
    }
}

/// Line segment (two endpoints)
#[derive(Clone, Copy, Debug)]
pub struct Segment {
    pub start: Point3,
    pub end: Point3,
}

impl Segment {
    pub fn new(start: Point3, end: Point3) -> Self {
        Self { start, end }
    }

    #[inline]
    pub fn length(&self) -> f32 {
        self.start.distance(self.end)
    }

    #[inline]
    pub fn midpoint(&self) -> Point3 {
        self.start.midpoint(self.end)
    }

    #[inline]
    pub fn direction(&self) -> Vector3 {
        self.end - self.start
    }

    /// Point at parameter t (0 = start, 1 = end)
    #[inline]
    pub fn point_at(&self, t: f32) -> Point3 {
        self.start.lerp(self.end, t)
    }

    /// Convert to infinite line
    pub fn to_line(&self) -> Option<Line> {
        Line::from_points(self.start, self.end)
    }
}

/// Axis-aligned bounding box in 3D
#[derive(Clone, Copy, Debug)]
pub struct BoundingBox3 {
    pub min: Point3,
    pub max: Point3,
}

impl BoundingBox3 {
    pub const EMPTY: Self = Self {
        min: Point3 { x: f32::INFINITY, y: f32::INFINITY, z: f32::INFINITY },
        max: Point3 { x: f32::NEG_INFINITY, y: f32::NEG_INFINITY, z: f32::NEG_INFINITY },
    };

    pub fn new(min: Point3, max: Point3) -> Self {
        Self {
            min: Point3::new(min.x.min(max.x), min.y.min(max.y), min.z.min(max.z)),
            max: Point3::new(min.x.max(max.x), min.y.max(max.y), min.z.max(max.z)),
        }
    }

    pub fn from_points(points: &[Point3]) -> Self {
        let mut bbox = Self::EMPTY;
        for &p in points {
            bbox = bbox.expand_by_point(p);
        }
        bbox
    }

    #[inline]
    pub fn center(&self) -> Point3 {
        self.min.midpoint(self.max)
    }

    #[inline]
    pub fn size(&self) -> Vector3 {
        self.max - self.min
    }

    #[inline]
    pub fn diagonal(&self) -> f32 {
        self.size().length()
    }

    pub fn expand_by_point(self, point: Point3) -> Self {
        Self {
            min: Point3::new(
                self.min.x.min(point.x),
                self.min.y.min(point.y),
                self.min.z.min(point.z),
            ),
            max: Point3::new(
                self.max.x.max(point.x),
                self.max.y.max(point.y),
                self.max.z.max(point.z),
            ),
        }
    }

    pub fn union(self, other: Self) -> Self {
        Self::new(
            Point3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            Point3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        )
    }

    pub fn contains(&self, point: Point3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x
            && point.y >= self.min.y && point.y <= self.max.y
            && point.z >= self.min.z && point.z <= self.max.z
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x
            && self.min.y <= other.max.y && self.max.y >= other.min.y
            && self.min.z <= other.max.z && self.max.z >= other.min.z
    }

    /// Expand box by margin on all sides
    pub fn expand(&self, margin: f32) -> Self {
        Self {
            min: Point3::new(self.min.x - margin, self.min.y - margin, self.min.z - margin),
            max: Point3::new(self.max.x + margin, self.max.y + margin, self.max.z + margin),
        }
    }
}

/// Affine transformation matrix (4x4)
#[derive(Clone, Copy, Debug)]
pub struct Transform3(pub Mat4);

impl Transform3 {
    pub const IDENTITY: Self = Self(Mat4::IDENTITY);

    #[inline]
    pub fn from_translation(v: Vector3) -> Self {
        Self(Mat4::from_translation(v.to_vec3()))
    }

    #[inline]
    pub fn from_scale(v: Vector3) -> Self {
        Self(Mat4::from_scale(v.to_vec3()))
    }

    #[inline]
    pub fn from_rotation_x(angle: f32) -> Self {
        Self(Mat4::from_rotation_x(angle))
    }

    #[inline]
    pub fn from_rotation_y(angle: f32) -> Self {
        Self(Mat4::from_rotation_y(angle))
    }

    #[inline]
    pub fn from_rotation_z(angle: f32) -> Self {
        Self(Mat4::from_rotation_z(angle))
    }

    #[inline]
    pub fn from_axis_angle(axis: Vector3, angle: f32) -> Self {
        Self(Mat4::from_axis_angle(axis.to_vec3(), angle))
    }

    /// Combine transformations (self applied first, then other)
    #[inline]
    pub fn then(self, other: Self) -> Self {
        Self(other.0 * self.0)
    }

    /// Inverse transformation
    pub fn inverse(self) -> Self {
        Self(self.0.inverse())
    }
}

impl Default for Transform3 {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_distance() {
        let p1 = Point3::new(0.0, 0.0, 0.0);
        let p2 = Point3::new(3.0, 4.0, 0.0);
        assert!((p1.distance(p2) - 5.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_vector_normalize() {
        let v = Vector3::new(3.0, 4.0, 0.0);
        let n = v.normalize().unwrap();
        assert!((n.length() - 1.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_vector_cross() {
        let v1 = Vector3::X;
        let v2 = Vector3::Y;
        let cross = v1.cross(v2);
        assert!((cross.x - Vector3::Z.x).abs() < TOLERANCE);
        assert!((cross.y - Vector3::Z.y).abs() < TOLERANCE);
        assert!((cross.z - Vector3::Z.z).abs() < TOLERANCE);
    }

    #[test]
    fn test_plane_distance() {
        let plane = Plane::XY;
        let point = Point3::new(1.0, 2.0, 5.0);
        assert!((plane.signed_distance(point) - 5.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_ray_plane_intersection() {
        let ray = Ray::new(Point3::new(0.0, 0.0, 5.0), Vector3::NEG_Z).unwrap();
        let plane = Plane::XY;
        let t = ray.intersect_plane(&plane).unwrap();
        assert!((t - 5.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_bounding_box() {
        let points = vec![
            Point3::new(1.0, 2.0, 3.0),
            Point3::new(-1.0, -2.0, -3.0),
            Point3::new(0.0, 0.0, 0.0),
        ];
        let bbox = BoundingBox3::from_points(&points);
        assert!(bbox.contains(Point3::ORIGIN));
        assert!(!bbox.contains(Point3::new(10.0, 0.0, 0.0)));
    }

    #[test]
    fn test_transform() {
        let p = Point3::new(1.0, 0.0, 0.0);
        let t = Transform3::from_translation(Vector3::new(0.0, 5.0, 0.0));
        let transformed = p.transform(&t);
        assert!((transformed.y - 5.0).abs() < TOLERANCE);
    }
}
