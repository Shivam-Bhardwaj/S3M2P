//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: mod.rs | DNA/src/cad/mod.rs
//! PURPOSE: B-Rep (Boundary Representation) CAD kernel for solid modeling
//! MODIFIED: 2025-12-09
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════

//!
//! PURPOSE: B-Rep (Boundary Representation) CAD kernel for solid modeling
//!
//! LAYER: DNA → CAD
//!
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │ MODULE STRUCTURE                                                            │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │   cad/                                                                      │
//! │   ├── geometry.rs    3D geometry primitives (Point3, Vector3, Plane, etc.) │
//! │   ├── topology.rs    B-Rep topology (Vertex, Edge, Face, Shell, Solid)     │
//! │   └── primitives.rs  Solid generators (box, cylinder, sphere, cone)        │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//!
//! DEPENDS ON:
//!   • glam → Vector/matrix math
//!
//! USED BY:
//!   • CORE/CAD_ENGINE   → High-level CAD operations
//!   • TOOLS/AUTOCRATE   → Shipping crate geometry
//!   • Future: MCAD apps → Solid modeling
//!
//! ═══════════════════════════════════════════════════════════════════════════════

pub mod geometry;
pub mod primitives;
pub mod topology;

// Re-export commonly used types
pub use geometry::{
    BoundingBox3, Line, Plane, Point3, Ray, Segment, Transform3, Vector3, TOLERANCE,
};
pub use primitives::{
    make_box, make_box_at, make_cone, make_cone_at, make_cylinder, make_cylinder_at, make_sphere,
    make_sphere_at,
};
pub use topology::{
    CurveType, Edge, EdgeId, Face, FaceId, FaceOrientation, Loop, Shell, ShellId, Solid,
    SurfaceType, Vertex, VertexId,
};
