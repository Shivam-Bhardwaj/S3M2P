//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: primitives.rs | DNA/src/cad/primitives.rs
//! PURPOSE: Solid primitive generators (box, cylinder, sphere, etc.)
//! MODIFIED: 2025-12-09
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════

//!
//! PURPOSE: Solid primitive generators (box, cylinder, sphere, etc.)
//!
//! LAYER: DNA → CAD
//!
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │ PRIMITIVES                                                                  │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │ make_box()       Axis-aligned rectangular solid                             │
//! │ make_cylinder()  Circular cylinder with flat ends                           │
//! │ make_sphere()    Sphere (triangulated or B-Rep)                             │
//! │ make_cone()      Circular cone with flat base                               │
//! │ make_prism()     Extruded polygon                                           │
//! └─────────────────────────────────────────────────────────────────────────────┘
//!
//! ═══════════════════════════════════════════════════════════════════════════════

use super::geometry::{Point3, Vector3};
use super::topology::{EdgeId, FaceId, Loop, Solid, SurfaceType, VertexId};

/// Create an axis-aligned box centered at origin
///
/// # Arguments
/// * `width` - Size along X axis
/// * `depth` - Size along Y axis
/// * `height` - Size along Z axis
///
/// # Returns
/// A valid B-Rep solid representing the box
pub fn make_box(width: f32, depth: f32, height: f32) -> Solid {
    make_box_at(Point3::ORIGIN, width, depth, height)
}

/// Create an axis-aligned box at a specific position
///
/// # Arguments
/// * `center` - Center point of the box
/// * `width` - Size along X axis
/// * `depth` - Size along Y axis
/// * `height` - Size along Z axis
pub fn make_box_at(center: Point3, width: f32, depth: f32, height: f32) -> Solid {
    let mut solid = Solid::new();

    let hw = width / 2.0;
    let hd = depth / 2.0;
    let hh = height / 2.0;

    // Create 8 vertices
    // Bottom face (Z = -hh)
    let v0 = solid.add_vertex(Point3::new(center.x - hw, center.y - hd, center.z - hh));
    let v1 = solid.add_vertex(Point3::new(center.x + hw, center.y - hd, center.z - hh));
    let v2 = solid.add_vertex(Point3::new(center.x + hw, center.y + hd, center.z - hh));
    let v3 = solid.add_vertex(Point3::new(center.x - hw, center.y + hd, center.z - hh));

    // Top face (Z = +hh)
    let v4 = solid.add_vertex(Point3::new(center.x - hw, center.y - hd, center.z + hh));
    let v5 = solid.add_vertex(Point3::new(center.x + hw, center.y - hd, center.z + hh));
    let v6 = solid.add_vertex(Point3::new(center.x + hw, center.y + hd, center.z + hh));
    let v7 = solid.add_vertex(Point3::new(center.x - hw, center.y + hd, center.z + hh));

    // Create 12 edges
    // Bottom face edges
    let e_b0 = solid.add_edge(v0, v1);
    let e_b1 = solid.add_edge(v1, v2);
    let e_b2 = solid.add_edge(v2, v3);
    let e_b3 = solid.add_edge(v3, v0);

    // Top face edges
    let e_t0 = solid.add_edge(v4, v5);
    let e_t1 = solid.add_edge(v5, v6);
    let e_t2 = solid.add_edge(v6, v7);
    let e_t3 = solid.add_edge(v7, v4);

    // Vertical edges
    let e_v0 = solid.add_edge(v0, v4);
    let e_v1 = solid.add_edge(v1, v5);
    let e_v2 = solid.add_edge(v2, v6);
    let e_v3 = solid.add_edge(v3, v7);

    // Create 6 faces
    // Bottom face (normal -Z)
    let f_bottom = solid.add_face(SurfaceType::Planar {
        normal: Vector3::NEG_Z,
    });
    add_quad_loop(
        &mut solid,
        f_bottom,
        [e_b0, e_b1, e_b2, e_b3],
        [false, false, false, false],
    );

    // Top face (normal +Z)
    let f_top = solid.add_face(SurfaceType::Planar { normal: Vector3::Z });
    add_quad_loop(
        &mut solid,
        f_top,
        [e_t0, e_t1, e_t2, e_t3],
        [true, true, true, true],
    );

    // Front face (normal -Y)
    let f_front = solid.add_face(SurfaceType::Planar {
        normal: Vector3::NEG_Y,
    });
    add_quad_loop(
        &mut solid,
        f_front,
        [e_b0, e_v1, e_t0, e_v0],
        [true, true, false, false],
    );

    // Back face (normal +Y)
    let f_back = solid.add_face(SurfaceType::Planar { normal: Vector3::Y });
    add_quad_loop(
        &mut solid,
        f_back,
        [e_b2, e_v3, e_t2, e_v2],
        [true, true, false, false],
    );

    // Left face (normal -X)
    let f_left = solid.add_face(SurfaceType::Planar {
        normal: Vector3::NEG_X,
    });
    add_quad_loop(
        &mut solid,
        f_left,
        [e_b3, e_v0, e_t3, e_v3],
        [true, true, false, false],
    );

    // Right face (normal +X)
    let f_right = solid.add_face(SurfaceType::Planar { normal: Vector3::X });
    add_quad_loop(
        &mut solid,
        f_right,
        [e_b1, e_v2, e_t1, e_v1],
        [true, true, false, false],
    );

    // Create shell
    let shell = solid.add_shell();
    if let Some(s) = solid.shells.iter_mut().find(|s| s.id == shell) {
        s.faces = vec![f_bottom, f_top, f_front, f_back, f_left, f_right];
        s.is_closed = true;
    }

    // Assign faces to shell
    for face_id in [f_bottom, f_top, f_front, f_back, f_left, f_right] {
        if let Some(f) = solid.face_mut(face_id) {
            f.shell = Some(shell);
        }
    }

    solid
}

/// Helper to add a quad loop to a face
fn add_quad_loop(solid: &mut Solid, face_id: FaceId, edges: [EdgeId; 4], directions: [bool; 4]) {
    if let Some(face) = solid.face_mut(face_id) {
        let mut loop_ = Loop::new();
        for (edge, dir) in edges.iter().zip(directions.iter()) {
            loop_.add_edge(*edge, *dir);
        }
        face.outer_loop = loop_;
    }
}

/// Create a cylinder along Z axis centered at origin
///
/// # Arguments
/// * `radius` - Radius of the cylinder
/// * `height` - Height of the cylinder
/// * `segments` - Number of segments for approximation (min 3)
pub fn make_cylinder(radius: f32, height: f32, segments: u32) -> Solid {
    make_cylinder_at(Point3::ORIGIN, radius, height, segments)
}

/// Create a cylinder at a specific position
pub fn make_cylinder_at(center: Point3, radius: f32, height: f32, segments: u32) -> Solid {
    let mut solid = Solid::new();
    let segments = segments.max(3);
    let hh = height / 2.0;

    // Create vertices for top and bottom circles
    let mut bottom_verts = Vec::with_capacity(segments as usize);
    let mut top_verts = Vec::with_capacity(segments as usize);

    for i in 0..segments {
        let angle = 2.0 * std::f32::consts::PI * (i as f32) / (segments as f32);
        let x = center.x + radius * angle.cos();
        let y = center.y + radius * angle.sin();

        bottom_verts.push(solid.add_vertex(Point3::new(x, y, center.z - hh)));
        top_verts.push(solid.add_vertex(Point3::new(x, y, center.z + hh)));
    }

    // Create edges for bottom circle
    let mut bottom_edges = Vec::with_capacity(segments as usize);
    for i in 0..segments as usize {
        let next = (i + 1) % segments as usize;
        bottom_edges.push(solid.add_edge(bottom_verts[i], bottom_verts[next]));
    }

    // Create edges for top circle
    let mut top_edges = Vec::with_capacity(segments as usize);
    for i in 0..segments as usize {
        let next = (i + 1) % segments as usize;
        top_edges.push(solid.add_edge(top_verts[i], top_verts[next]));
    }

    // Create vertical edges
    let mut vert_edges = Vec::with_capacity(segments as usize);
    for i in 0..segments as usize {
        vert_edges.push(solid.add_edge(bottom_verts[i], top_verts[i]));
    }

    // Create bottom face
    let f_bottom = solid.add_face(SurfaceType::Planar {
        normal: Vector3::NEG_Z,
    });
    if let Some(face) = solid.face_mut(f_bottom) {
        let mut loop_ = Loop::new();
        for &edge in &bottom_edges {
            loop_.add_edge(edge, false); // Counter-clockwise when viewed from below
        }
        face.outer_loop = loop_;
    }

    // Create top face
    let f_top = solid.add_face(SurfaceType::Planar { normal: Vector3::Z });
    if let Some(face) = solid.face_mut(f_top) {
        let mut loop_ = Loop::new();
        for &edge in &top_edges {
            loop_.add_edge(edge, true); // Clockwise when viewed from above
        }
        face.outer_loop = loop_;
    }

    // Create cylindrical surface as multiple planar faces (approximation)
    let mut side_faces = Vec::with_capacity(segments as usize);
    for i in 0..segments as usize {
        let next = (i + 1) % segments as usize;

        // Compute face normal (average of vertex normals)
        let v1 = solid.vertex(bottom_verts[i]).unwrap().point;
        let v2 = solid.vertex(bottom_verts[next]).unwrap().point;
        let mid = v1.midpoint(v2);
        let normal = Vector3::new(mid.x - center.x, mid.y - center.y, 0.0)
            .normalize()
            .unwrap_or(Vector3::X);

        let f_side = solid.add_face(SurfaceType::Planar { normal });
        if let Some(face) = solid.face_mut(f_side) {
            let mut loop_ = Loop::new();
            loop_.add_edge(bottom_edges[i], true);
            loop_.add_edge(vert_edges[next], true);
            loop_.add_edge(top_edges[i], false);
            loop_.add_edge(vert_edges[i], false);
            face.outer_loop = loop_;
        }
        side_faces.push(f_side);
    }

    // Create shell
    let shell = solid.add_shell();
    if let Some(s) = solid.shells.iter_mut().find(|s| s.id == shell) {
        s.faces.push(f_bottom);
        s.faces.push(f_top);
        s.faces.extend(side_faces.iter().cloned());
        s.is_closed = true;
    }

    solid
}

/// Create a sphere centered at origin
///
/// # Arguments
/// * `radius` - Radius of the sphere
/// * `u_segments` - Longitude divisions (min 4)
/// * `v_segments` - Latitude divisions (min 2)
pub fn make_sphere(radius: f32, u_segments: u32, v_segments: u32) -> Solid {
    make_sphere_at(Point3::ORIGIN, radius, u_segments, v_segments)
}

/// Create a sphere at a specific position
pub fn make_sphere_at(center: Point3, radius: f32, u_segments: u32, v_segments: u32) -> Solid {
    let mut solid = Solid::new();
    let u_segments = u_segments.max(4);
    let v_segments = v_segments.max(2);

    // Create vertices
    // Top pole
    let top_pole = solid.add_vertex(Point3::new(center.x, center.y, center.z + radius));

    // Middle rings
    let mut rings: Vec<Vec<VertexId>> = Vec::with_capacity(v_segments as usize - 1);
    for j in 1..v_segments {
        let phi = std::f32::consts::PI * (j as f32) / (v_segments as f32);
        let z = center.z + radius * phi.cos();
        let ring_radius = radius * phi.sin();

        let mut ring = Vec::with_capacity(u_segments as usize);
        for i in 0..u_segments {
            let theta = 2.0 * std::f32::consts::PI * (i as f32) / (u_segments as f32);
            let x = center.x + ring_radius * theta.cos();
            let y = center.y + ring_radius * theta.sin();
            ring.push(solid.add_vertex(Point3::new(x, y, z)));
        }
        rings.push(ring);
    }

    // Bottom pole
    let bottom_pole = solid.add_vertex(Point3::new(center.x, center.y, center.z - radius));

    // Create faces
    // Top cap (triangles connecting top pole to first ring)
    if !rings.is_empty() {
        let first_ring = &rings[0];
        for i in 0..u_segments as usize {
            let next = (i + 1) % u_segments as usize;

            let e1 = solid.add_edge(top_pole, first_ring[i]);
            let e2 = solid.add_edge(first_ring[i], first_ring[next]);
            let e3 = solid.add_edge(first_ring[next], top_pole);

            let f = solid.add_face(SurfaceType::Spherical { center, radius });
            if let Some(face) = solid.face_mut(f) {
                let mut loop_ = Loop::new();
                loop_.add_edge(e1, true);
                loop_.add_edge(e2, true);
                loop_.add_edge(e3, true);
                face.outer_loop = loop_;
            }
        }
    }

    // Middle bands (quads between adjacent rings)
    for j in 0..rings.len().saturating_sub(1) {
        let ring1 = &rings[j];
        let ring2 = &rings[j + 1];

        for i in 0..u_segments as usize {
            let next = (i + 1) % u_segments as usize;

            let e1 = solid.add_edge(ring1[i], ring1[next]);
            let e2 = solid.add_edge(ring1[next], ring2[next]);
            let e3 = solid.add_edge(ring2[next], ring2[i]);
            let e4 = solid.add_edge(ring2[i], ring1[i]);

            let f = solid.add_face(SurfaceType::Spherical { center, radius });
            if let Some(face) = solid.face_mut(f) {
                let mut loop_ = Loop::new();
                loop_.add_edge(e1, true);
                loop_.add_edge(e2, true);
                loop_.add_edge(e3, true);
                loop_.add_edge(e4, true);
                face.outer_loop = loop_;
            }
        }
    }

    // Bottom cap (triangles connecting last ring to bottom pole)
    if !rings.is_empty() {
        let last_ring = rings.last().unwrap();
        for i in 0..u_segments as usize {
            let next = (i + 1) % u_segments as usize;

            let e1 = solid.add_edge(last_ring[i], last_ring[next]);
            let e2 = solid.add_edge(last_ring[next], bottom_pole);
            let e3 = solid.add_edge(bottom_pole, last_ring[i]);

            let f = solid.add_face(SurfaceType::Spherical { center, radius });
            if let Some(face) = solid.face_mut(f) {
                let mut loop_ = Loop::new();
                loop_.add_edge(e1, true);
                loop_.add_edge(e2, true);
                loop_.add_edge(e3, true);
                face.outer_loop = loop_;
            }
        }
    }

    // Create shell
    let shell = solid.add_shell();
    if let Some(s) = solid.shells.iter_mut().find(|s| s.id == shell) {
        s.faces = solid.faces.iter().map(|f| f.id).collect();
        s.is_closed = true;
    }

    solid
}

/// Create a cone along Z axis with apex at top
///
/// # Arguments
/// * `base_radius` - Radius at the base
/// * `height` - Height of the cone
/// * `segments` - Number of segments for approximation (min 3)
pub fn make_cone(base_radius: f32, height: f32, segments: u32) -> Solid {
    make_cone_at(Point3::ORIGIN, base_radius, height, segments)
}

/// Create a cone at a specific position (center of base)
pub fn make_cone_at(base_center: Point3, base_radius: f32, height: f32, segments: u32) -> Solid {
    let mut solid = Solid::new();
    let segments = segments.max(3);

    // Apex vertex
    let apex = solid.add_vertex(Point3::new(
        base_center.x,
        base_center.y,
        base_center.z + height,
    ));

    // Base circle vertices
    let mut base_verts = Vec::with_capacity(segments as usize);
    for i in 0..segments {
        let angle = 2.0 * std::f32::consts::PI * (i as f32) / (segments as f32);
        let x = base_center.x + base_radius * angle.cos();
        let y = base_center.y + base_radius * angle.sin();
        base_verts.push(solid.add_vertex(Point3::new(x, y, base_center.z)));
    }

    // Create base edges
    let mut base_edges = Vec::with_capacity(segments as usize);
    for i in 0..segments as usize {
        let next = (i + 1) % segments as usize;
        base_edges.push(solid.add_edge(base_verts[i], base_verts[next]));
    }

    // Create side edges (from apex to base)
    let mut side_edges = Vec::with_capacity(segments as usize);
    for vert in &base_verts {
        side_edges.push(solid.add_edge(apex, *vert));
    }

    // Create base face
    let f_base = solid.add_face(SurfaceType::Planar {
        normal: Vector3::NEG_Z,
    });
    if let Some(face) = solid.face_mut(f_base) {
        let mut loop_ = Loop::new();
        for &edge in &base_edges {
            loop_.add_edge(edge, false);
        }
        face.outer_loop = loop_;
    }

    // Create triangular side faces
    for i in 0..segments as usize {
        let next = (i + 1) % segments as usize;

        // Normal points outward (cross product of two edges)
        let v1 = solid.vertex(base_verts[i]).unwrap().point;
        let v2 = solid.vertex(base_verts[next]).unwrap().point;
        let mid = v1.midpoint(v2);
        let _normal = Vector3::new(mid.x - base_center.x, mid.y - base_center.y, 0.0)
            .normalize()
            .unwrap_or(Vector3::X);

        let f_side = solid.add_face(SurfaceType::Conical {
            apex: Point3::new(base_center.x, base_center.y, base_center.z + height),
            axis: Vector3::Z,
            half_angle: (base_radius / height).atan(),
        });
        if let Some(face) = solid.face_mut(f_side) {
            let mut loop_ = Loop::new();
            loop_.add_edge(side_edges[i], false);
            loop_.add_edge(base_edges[i], true);
            loop_.add_edge(side_edges[next], true);
            face.outer_loop = loop_;
        }
    }

    // Create shell
    let shell = solid.add_shell();
    if let Some(s) = solid.shells.iter_mut().find(|s| s.id == shell) {
        s.faces = solid.faces.iter().map(|f| f.id).collect();
        s.is_closed = true;
    }

    solid
}

#[cfg(test)]
mod tests {
    use super::super::geometry::TOLERANCE;
    use super::*;

    #[test]
    fn test_make_box() {
        let solid = make_box(2.0, 3.0, 4.0);

        // Should have 8 vertices
        assert_eq!(solid.vertices.len(), 8);

        // Should have 12 edges
        assert_eq!(solid.edges.len(), 12);

        // Should have 6 faces
        assert_eq!(solid.faces.len(), 6);

        // Should have 1 closed shell
        assert_eq!(solid.shells.len(), 1);
        assert!(solid.shells[0].is_closed);
    }

    #[test]
    fn test_make_cylinder() {
        let solid = make_cylinder(1.0, 2.0, 8);

        // Should have vertices for top and bottom circles
        assert_eq!(solid.vertices.len(), 16); // 8 * 2

        // Should be valid
        assert!(solid.is_valid());
    }

    #[test]
    fn test_make_sphere() {
        let solid = make_sphere(1.0, 8, 4);

        // Should have poles plus ring vertices
        // 2 poles + (4-1) rings * 8 vertices = 2 + 24 = 26
        assert_eq!(solid.vertices.len(), 26);

        // Should be valid
        assert!(solid.is_valid());
    }

    #[test]
    fn test_make_cone() {
        let solid = make_cone(1.0, 2.0, 6);

        // 1 apex + 6 base vertices
        assert_eq!(solid.vertices.len(), 7);

        // 6 base edges + 6 side edges
        assert_eq!(solid.edges.len(), 12);

        // 1 base face + 6 side faces
        assert_eq!(solid.faces.len(), 7);

        assert!(solid.is_valid());
    }

    #[test]
    fn test_box_bounding_box() {
        let mut solid = make_box(4.0, 6.0, 8.0);
        let bbox = solid.bounding_box();

        assert!((bbox.size().x - 4.0).abs() < TOLERANCE);
        assert!((bbox.size().y - 6.0).abs() < TOLERANCE);
        assert!((bbox.size().z - 8.0).abs() < TOLERANCE);
    }
}
