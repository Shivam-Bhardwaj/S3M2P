//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: topology.rs
//! PATH: DNA/src/cad/topology.rs
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! PURPOSE: B-Rep topology primitives (Vertex, Edge, Face, Shell, Solid)
//!
//! LAYER: DNA → CAD
//!
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │ B-REP TOPOLOGY HIERARCHY                                                    │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │   Solid ─────────────────────────────────────────────────────────────────┐  │
//! │     │                                                                    │  │
//! │     └── Shell[] ─────────────────────────────────────────────────────┐   │  │
//! │           │                                                          │   │  │
//! │           └── Face[] ───────────────────────────────────────────┐    │   │  │
//! │                 │                                               │    │   │  │
//! │                 ├── Surface (geometry backing)                  │    │   │  │
//! │                 │                                               │    │   │  │
//! │                 └── Loop[] ─────────────────────────────────┐   │    │   │  │
//! │                       │                                     │   │    │   │  │
//! │                       └── HalfEdge[] ───────────────────┐   │   │    │   │  │
//! │                             │                           │   │   │    │   │  │
//! │                             ├── Edge (shared)           │   │   │    │   │  │
//! │                             │     └── Curve (geometry)  │   │   │    │   │  │
//! │                             │                           │   │   │    │   │  │
//! │                             └── Vertex (at endpoints)   │   │   │    │   │  │
//! │                                   └── Point3 (geometry) │   │   │    │   │  │
//! │                                                         │   │   │    │   │  │
//! └─────────────────────────────────────────────────────────┴───┴───┴────┴───┴──┘
//!
//! DEPENDS ON:
//!   • DNA/src/cad/geometry.rs → Point3, Vector3, etc.
//!
//! USED BY:
//!   • CORE/CAD_ENGINE → Solid modeling operations
//!
//! ═══════════════════════════════════════════════════════════════════════════════

use super::geometry::{BoundingBox3, Point3, Segment, Vector3, TOLERANCE};

/// Handle to a vertex in a B-Rep structure
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VertexId(pub u32);

/// Handle to an edge in a B-Rep structure
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EdgeId(pub u32);

/// Handle to a face in a B-Rep structure
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FaceId(pub u32);

/// Handle to a shell in a B-Rep structure
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ShellId(pub u32);

/// Topological vertex - represents a point in space
#[derive(Clone, Debug)]
pub struct Vertex {
    pub id: VertexId,
    pub point: Point3,
    /// Edges that start or end at this vertex
    pub edges: Vec<EdgeId>,
}

impl Vertex {
    pub fn new(id: VertexId, point: Point3) -> Self {
        Self {
            id,
            point,
            edges: Vec::new(),
        }
    }

    /// Check if this vertex is at the same location as another
    pub fn coincident(&self, other: &Vertex) -> bool {
        self.point.approx_eq(other.point, TOLERANCE)
    }
}

/// Curve type for edge geometry
#[derive(Clone, Debug)]
pub enum CurveType {
    /// Straight line segment
    Linear,
    /// Circular arc (center, radius, start_angle, end_angle)
    Arc {
        center: Point3,
        radius: f32,
        normal: Vector3,
        start_angle: f32,
        end_angle: f32,
    },
    /// NURBS curve (control points, weights, knots, degree)
    Nurbs {
        control_points: Vec<Point3>,
        weights: Vec<f32>,
        knots: Vec<f32>,
        degree: u32,
    },
}

impl CurveType {
    /// Evaluate point on curve at parameter t (0..1)
    pub fn point_at(&self, start: Point3, end: Point3, t: f32) -> Point3 {
        match self {
            CurveType::Linear => start.lerp(end, t),
            CurveType::Arc {
                center,
                radius,
                normal,
                start_angle,
                end_angle,
            } => {
                let angle = start_angle + t * (end_angle - start_angle);
                // Compute point on arc (simplified - assumes XY plane arc)
                let u = if normal.z.abs() > 0.9 {
                    Vector3::X
                } else {
                    Vector3::Z.cross(*normal).normalize_or_z()
                };
                let v = normal.cross(u);
                let x = center.x + radius * (angle.cos() * u.x + angle.sin() * v.x);
                let y = center.y + radius * (angle.cos() * u.y + angle.sin() * v.y);
                let z = center.z + radius * (angle.cos() * u.z + angle.sin() * v.z);
                Point3::new(x, y, z)
            }
            CurveType::Nurbs {
                control_points,
                weights,
                knots,
                degree,
            } => {
                // De Boor's algorithm for NURBS evaluation
                nurbs_eval(control_points, weights, knots, *degree, t)
            }
        }
    }
}

/// Evaluate NURBS curve at parameter t using De Boor's algorithm
fn nurbs_eval(
    control_points: &[Point3],
    weights: &[f32],
    knots: &[f32],
    degree: u32,
    t: f32,
) -> Point3 {
    let n = control_points.len();
    if n == 0 {
        return Point3::ORIGIN;
    }

    // Find knot span
    let mut span = degree as usize;
    for i in (degree as usize)..n {
        if t >= knots[i] && t < knots[i + 1] {
            span = i;
            break;
        }
    }

    // De Boor's algorithm
    let mut d: Vec<(Point3, f32)> = (0..=degree as usize)
        .map(|j| {
            let idx = span - degree as usize + j;
            (control_points[idx.min(n - 1)], weights[idx.min(n - 1)])
        })
        .collect();

    for r in 1..=degree as usize {
        for j in (r..=degree as usize).rev() {
            let i = span - degree as usize + j;
            let denom = knots[i + degree as usize + 1 - r] - knots[i];
            let alpha = if denom.abs() < TOLERANCE {
                0.0
            } else {
                (t - knots[i]) / denom
            };

            let w = (1.0 - alpha) * d[j - 1].1 + alpha * d[j].1;
            let p = if w.abs() < TOLERANCE {
                d[j].0
            } else {
                let p1 = d[j - 1].0.to_vec3() * d[j - 1].1;
                let p2 = d[j].0.to_vec3() * d[j].1;
                Point3::from_vec3((p1 * (1.0 - alpha) + p2 * alpha) / w)
            };
            d[j] = (p, w);
        }
    }

    d[degree as usize].0
}

/// Topological edge - bounded curve between two vertices
#[derive(Clone, Debug)]
pub struct Edge {
    pub id: EdgeId,
    pub start: VertexId,
    pub end: VertexId,
    pub curve: CurveType,
    /// Faces that share this edge (usually 2 for manifold, 1 for boundary)
    pub faces: Vec<FaceId>,
}

impl Edge {
    pub fn new(id: EdgeId, start: VertexId, end: VertexId) -> Self {
        Self {
            id,
            start,
            end,
            curve: CurveType::Linear,
            faces: Vec::new(),
        }
    }

    pub fn with_curve(mut self, curve: CurveType) -> Self {
        self.curve = curve;
        self
    }

    /// Convert to line segment (only valid for linear edges)
    pub fn to_segment(&self, vertices: &[Vertex]) -> Option<Segment> {
        let start_v = vertices.iter().find(|v| v.id == self.start)?;
        let end_v = vertices.iter().find(|v| v.id == self.end)?;
        Some(Segment::new(start_v.point, end_v.point))
    }
}

/// Surface type for face geometry
#[derive(Clone, Debug)]
pub enum SurfaceType {
    /// Flat plane
    Planar { normal: Vector3 },
    /// Cylindrical surface
    Cylindrical {
        axis: Vector3,
        center: Point3,
        radius: f32,
    },
    /// Spherical surface
    Spherical { center: Point3, radius: f32 },
    /// Conical surface
    Conical {
        apex: Point3,
        axis: Vector3,
        half_angle: f32,
    },
    /// Toroidal surface
    Toroidal {
        center: Point3,
        axis: Vector3,
        major_radius: f32,
        minor_radius: f32,
    },
    /// NURBS surface
    Nurbs {
        control_points: Vec<Vec<Point3>>,
        weights: Vec<Vec<f32>>,
        u_knots: Vec<f32>,
        v_knots: Vec<f32>,
        u_degree: u32,
        v_degree: u32,
    },
}

impl SurfaceType {
    /// Get surface normal at a point (approximate for complex surfaces)
    pub fn normal_at(&self, _point: Point3) -> Vector3 {
        match self {
            SurfaceType::Planar { normal } => *normal,
            SurfaceType::Cylindrical {
                axis,
                center: _,
                radius: _,
            } => {
                // Normal points radially outward from axis
                *axis // Simplified - should compute radial direction
            }
            SurfaceType::Spherical {
                center: _,
                radius: _,
            } => {
                // Normal points radially outward from center
                Vector3::Z // Simplified - should compute from point to center
            }
            _ => Vector3::Z, // Placeholder for complex surfaces
        }
    }
}

/// Loop - closed sequence of half-edges bounding a face
#[derive(Clone, Debug)]
pub struct Loop {
    /// Edges in order around the loop
    pub edges: Vec<EdgeId>,
    /// Direction for each edge (true = forward, false = reverse)
    pub directions: Vec<bool>,
}

impl Loop {
    pub fn new() -> Self {
        Self {
            edges: Vec::new(),
            directions: Vec::new(),
        }
    }

    pub fn add_edge(&mut self, edge: EdgeId, forward: bool) {
        self.edges.push(edge);
        self.directions.push(forward);
    }

    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    pub fn len(&self) -> usize {
        self.edges.len()
    }
}

impl Default for Loop {
    fn default() -> Self {
        Self::new()
    }
}

/// Face orientation relative to shell
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaceOrientation {
    /// Normal points outward from shell
    Outward,
    /// Normal points inward (for voids/holes)
    Inward,
}

/// Topological face - bounded surface region
#[derive(Clone, Debug)]
pub struct Face {
    pub id: FaceId,
    pub surface: SurfaceType,
    /// Outer boundary loop
    pub outer_loop: Loop,
    /// Inner boundary loops (holes in the face)
    pub inner_loops: Vec<Loop>,
    pub orientation: FaceOrientation,
    /// Shell this face belongs to
    pub shell: Option<ShellId>,
}

impl Face {
    pub fn new(id: FaceId, surface: SurfaceType) -> Self {
        Self {
            id,
            surface,
            outer_loop: Loop::new(),
            inner_loops: Vec::new(),
            orientation: FaceOrientation::Outward,
            shell: None,
        }
    }

    pub fn with_outer_loop(mut self, loop_: Loop) -> Self {
        self.outer_loop = loop_;
        self
    }

    pub fn add_inner_loop(&mut self, loop_: Loop) {
        self.inner_loops.push(loop_);
    }

    /// Get all edges bounding this face
    pub fn all_edges(&self) -> Vec<EdgeId> {
        let mut edges = self.outer_loop.edges.clone();
        for inner in &self.inner_loops {
            edges.extend(inner.edges.iter().cloned());
        }
        edges
    }
}

/// Shell - connected set of faces forming a closed or open surface
#[derive(Clone, Debug)]
pub struct Shell {
    pub id: ShellId,
    pub faces: Vec<FaceId>,
    /// True if shell is closed (watertight)
    pub is_closed: bool,
}

impl Shell {
    pub fn new(id: ShellId) -> Self {
        Self {
            id,
            faces: Vec::new(),
            is_closed: false,
        }
    }

    pub fn add_face(&mut self, face: FaceId) {
        self.faces.push(face);
    }
}

/// Solid - collection of shells representing a 3D volume
#[derive(Clone, Debug, Default)]
pub struct Solid {
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,
    pub faces: Vec<Face>,
    pub shells: Vec<Shell>,
    /// Bounding box (cached)
    bounds: Option<BoundingBox3>,
}

impl Solid {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a vertex, returning its ID
    pub fn add_vertex(&mut self, point: Point3) -> VertexId {
        let id = VertexId(self.vertices.len() as u32);
        self.vertices.push(Vertex::new(id, point));
        self.bounds = None; // Invalidate cache
        id
    }

    /// Add an edge between two vertices
    pub fn add_edge(&mut self, start: VertexId, end: VertexId) -> EdgeId {
        let id = EdgeId(self.edges.len() as u32);
        self.edges.push(Edge::new(id, start, end));

        // Update vertex edge lists
        if let Some(v) = self.vertices.iter_mut().find(|v| v.id == start) {
            v.edges.push(id);
        }
        if let Some(v) = self.vertices.iter_mut().find(|v| v.id == end) {
            v.edges.push(id);
        }

        id
    }

    /// Add a face with given surface type
    pub fn add_face(&mut self, surface: SurfaceType) -> FaceId {
        let id = FaceId(self.faces.len() as u32);
        self.faces.push(Face::new(id, surface));
        id
    }

    /// Add a shell
    pub fn add_shell(&mut self) -> ShellId {
        let id = ShellId(self.shells.len() as u32);
        self.shells.push(Shell::new(id));
        id
    }

    /// Get bounding box of all vertices
    pub fn bounding_box(&mut self) -> BoundingBox3 {
        if let Some(bounds) = self.bounds {
            return bounds;
        }

        let points: Vec<Point3> = self.vertices.iter().map(|v| v.point).collect();
        let bounds = BoundingBox3::from_points(&points);
        self.bounds = Some(bounds);
        bounds
    }

    /// Get vertex by ID
    pub fn vertex(&self, id: VertexId) -> Option<&Vertex> {
        self.vertices.iter().find(|v| v.id == id)
    }

    /// Get edge by ID
    pub fn edge(&self, id: EdgeId) -> Option<&Edge> {
        self.edges.iter().find(|e| e.id == id)
    }

    /// Get face by ID
    pub fn face(&self, id: FaceId) -> Option<&Face> {
        self.faces.iter().find(|f| f.id == id)
    }

    /// Get mutable face by ID
    pub fn face_mut(&mut self, id: FaceId) -> Option<&mut Face> {
        self.faces.iter_mut().find(|f| f.id == id)
    }

    /// Check if solid is valid (basic topology checks)
    pub fn is_valid(&self) -> bool {
        // Check all edges reference valid vertices
        for edge in &self.edges {
            if self.vertex(edge.start).is_none() || self.vertex(edge.end).is_none() {
                return false;
            }
        }

        // Check all face loops reference valid edges
        for face in &self.faces {
            for edge_id in &face.outer_loop.edges {
                if self.edge(*edge_id).is_none() {
                    return false;
                }
            }
            for inner in &face.inner_loops {
                for edge_id in &inner.edges {
                    if self.edge(*edge_id).is_none() {
                        return false;
                    }
                }
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_vertex() {
        let v = Vertex::new(VertexId(0), Point3::new(1.0, 2.0, 3.0));
        assert_eq!(v.id, VertexId(0));
        assert!((v.point.x - 1.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_solid_add_vertex() {
        let mut solid = Solid::new();
        let v1 = solid.add_vertex(Point3::new(0.0, 0.0, 0.0));
        let v2 = solid.add_vertex(Point3::new(1.0, 0.0, 0.0));

        assert_eq!(v1, VertexId(0));
        assert_eq!(v2, VertexId(1));
        assert_eq!(solid.vertices.len(), 2);
    }

    #[test]
    fn test_solid_add_edge() {
        let mut solid = Solid::new();
        let v1 = solid.add_vertex(Point3::new(0.0, 0.0, 0.0));
        let v2 = solid.add_vertex(Point3::new(1.0, 0.0, 0.0));
        let e1 = solid.add_edge(v1, v2);

        assert_eq!(e1, EdgeId(0));
        assert_eq!(solid.edges.len(), 1);

        // Check vertex edge lists updated
        assert!(solid.vertex(v1).unwrap().edges.contains(&e1));
        assert!(solid.vertex(v2).unwrap().edges.contains(&e1));
    }

    #[test]
    fn test_linear_curve() {
        let curve = CurveType::Linear;
        let start = Point3::new(0.0, 0.0, 0.0);
        let end = Point3::new(10.0, 0.0, 0.0);

        let mid = curve.point_at(start, end, 0.5);
        assert!((mid.x - 5.0).abs() < TOLERANCE);
    }

    #[test]
    fn test_loop() {
        let mut loop_ = Loop::new();
        loop_.add_edge(EdgeId(0), true);
        loop_.add_edge(EdgeId(1), true);
        loop_.add_edge(EdgeId(2), false);

        assert_eq!(loop_.len(), 3);
        assert_eq!(loop_.directions[2], false);
    }

    #[test]
    fn test_solid_validity() {
        let mut solid = Solid::new();
        let v1 = solid.add_vertex(Point3::new(0.0, 0.0, 0.0));
        let v2 = solid.add_vertex(Point3::new(1.0, 0.0, 0.0));
        let v3 = solid.add_vertex(Point3::new(0.5, 1.0, 0.0));

        solid.add_edge(v1, v2);
        solid.add_edge(v2, v3);
        solid.add_edge(v3, v1);

        assert!(solid.is_valid());
    }
}
