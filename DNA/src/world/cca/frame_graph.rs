//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: frame_graph.rs | DNA/src/world/cca/frame_graph.rs
//! PURPOSE: Reference frame graph with SE(3) transformations
//! CREATED: 2025-12-09
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! # Frame Graph
//!
//! A directed graph where:
//! - Nodes = Reference frames (HCI, HEE, GCI, GSE, mission frames)
//! - Edges = SE(3) transformations between frames
//!
//! The graph automatically finds transformation paths and composes them.
//!
//! ## Usage
//!
//! ```ignore
//! let mut graph = FrameGraph::with_builtins();
//! let epoch = Epoch::j2000();
//!
//! // Transform position from HCI to GSE
//! let pos_hci = DVec3::new(1.0, 0.0, 0.0);
//! let pos_gse = graph.transform(pos_hci, FrameId::HCI, FrameId::GSE, epoch);
//! ```
//!
//! ═══════════════════════════════════════════════════════════════════════════════

use super::epoch::Epoch;
use super::point::ConformalPoint;
use super::se3::Se3;
use glam::{DMat4, DVec3};
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────────────────────
// FRAME IDENTIFIER
// ─────────────────────────────────────────────────────────────────────────────────

/// Unique identifier for a reference frame
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FrameId(pub u32);

impl FrameId {
    // Built-in frame IDs
    /// International Celestial Reference Frame (barycentric, J2000)
    pub const ICRF: FrameId = FrameId(0);
    /// Heliocentric Inertial (Sun-centered, J2000 ecliptic)
    pub const HCI: FrameId = FrameId(1);
    /// Heliocentric Earth Ecliptic (X toward Earth)
    pub const HEE: FrameId = FrameId(2);
    /// Heliocentric Aries Ecliptic (X toward vernal equinox)
    pub const HAE: FrameId = FrameId(3);
    /// Geocentric Inertial (Earth-centered, J2000 equator)
    pub const GCI: FrameId = FrameId(4);
    /// Geocentric Solar Ecliptic (X toward Sun)
    pub const GSE: FrameId = FrameId(5);
    /// Geocentric Solar Magnetospheric (X toward Sun, Z toward dipole)
    pub const GSM: FrameId = FrameId(6);
    /// Radial-Tangential-Normal (spacecraft-centered)
    pub const RTN: FrameId = FrameId(7);

    /// First user-defined frame ID
    pub const USER_START: u32 = 1000;
}

// ─────────────────────────────────────────────────────────────────────────────────
// FRAME DEFINITION
// ─────────────────────────────────────────────────────────────────────────────────

/// Definition of a reference frame
#[derive(Clone)]
pub struct FrameDef {
    /// Unique identifier
    pub id: FrameId,
    /// Human-readable name
    pub name: &'static str,
    /// Description
    pub description: &'static str,
    /// Center body (if any)
    pub center: Option<CelestialBody>,
    /// Parent frame for transformations
    pub parent: Option<FrameId>,
}

/// Celestial body that can be a frame center
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CelestialBody {
    SolarSystemBarycenter,
    Sun,
    Mercury,
    Venus,
    Earth,
    Moon,
    Mars,
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
    Pluto,
    Spacecraft(u32), // Mission ID
}

// ─────────────────────────────────────────────────────────────────────────────────
// FRAME TRANSFORM
// ─────────────────────────────────────────────────────────────────────────────────

/// Transform function between two frames
pub type TransformFn = Box<dyn Fn(Epoch) -> Se3 + Send + Sync>;

/// Edge in the frame graph (transformation between frames)
pub struct FrameEdge {
    pub from: FrameId,
    pub to: FrameId,
    /// Time-dependent transformation function
    pub transform: TransformFn,
}

// ─────────────────────────────────────────────────────────────────────────────────
// FRAME GRAPH
// ─────────────────────────────────────────────────────────────────────────────────

/// Graph of reference frames with SE(3) transformations
pub struct FrameGraph {
    /// Frame definitions by ID
    frames: HashMap<FrameId, FrameDef>,
    /// Transformations between frames: (from, to) -> edge
    edges: HashMap<(FrameId, FrameId), FrameEdge>,
    /// Next available user frame ID
    next_user_id: u32,
}

impl FrameGraph {
    /// Create an empty frame graph
    pub fn new() -> Self {
        Self {
            frames: HashMap::new(),
            edges: HashMap::new(),
            next_user_id: FrameId::USER_START,
        }
    }

    /// Create a frame graph with built-in astronomical frames
    pub fn with_builtins() -> Self {
        let mut graph = Self::new();

        // Register built-in frames
        graph.register_frame(FrameDef {
            id: FrameId::ICRF,
            name: "ICRF",
            description: "International Celestial Reference Frame (J2000 barycentric)",
            center: Some(CelestialBody::SolarSystemBarycenter),
            parent: None,
        });

        graph.register_frame(FrameDef {
            id: FrameId::HCI,
            name: "HCI",
            description: "Heliocentric Inertial (J2000 ecliptic, Sun-centered)",
            center: Some(CelestialBody::Sun),
            parent: Some(FrameId::ICRF),
        });

        graph.register_frame(FrameDef {
            id: FrameId::HEE,
            name: "HEE",
            description: "Heliocentric Earth Ecliptic (X toward Earth)",
            center: Some(CelestialBody::Sun),
            parent: Some(FrameId::HCI),
        });

        graph.register_frame(FrameDef {
            id: FrameId::GCI,
            name: "GCI",
            description: "Geocentric Inertial (J2000 equator)",
            center: Some(CelestialBody::Earth),
            parent: Some(FrameId::HCI),
        });

        graph.register_frame(FrameDef {
            id: FrameId::GSE,
            name: "GSE",
            description: "Geocentric Solar Ecliptic (X toward Sun)",
            center: Some(CelestialBody::Earth),
            parent: Some(FrameId::GCI),
        });

        // Register built-in transforms
        graph.register_builtin_transforms();

        graph
    }

    /// Register a new frame
    pub fn register_frame(&mut self, def: FrameDef) {
        self.frames.insert(def.id, def);
    }

    /// Allocate a new user frame ID
    pub fn allocate_frame_id(&mut self) -> FrameId {
        let id = FrameId(self.next_user_id);
        self.next_user_id += 1;
        id
    }

    /// Register a transformation between two frames
    pub fn register_transform<F>(&mut self, from: FrameId, to: FrameId, transform: F)
    where
        F: Fn(Epoch) -> Se3 + Send + Sync + 'static,
    {
        self.edges.insert(
            (from, to),
            FrameEdge {
                from,
                to,
                transform: Box::new(transform),
            },
        );
    }

    /// Get frame definition by ID
    pub fn get_frame(&self, id: FrameId) -> Option<&FrameDef> {
        self.frames.get(&id)
    }

    /// Find transformation path between two frames using BFS
    pub fn find_path(&self, from: FrameId, to: FrameId) -> Option<Vec<FrameId>> {
        if from == to {
            return Some(vec![from]);
        }

        // BFS to find shortest path
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        let mut parent: HashMap<FrameId, FrameId> = HashMap::new();

        queue.push_back(from);
        visited.insert(from);

        while let Some(current) = queue.pop_front() {
            // Check all edges from current frame
            for &(edge_from, edge_to) in self.edges.keys() {
                let neighbor = if edge_from == current {
                    edge_to
                } else if edge_to == current {
                    edge_from // Allow reverse traversal
                } else {
                    continue;
                };

                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    parent.insert(neighbor, current);

                    if neighbor == to {
                        // Reconstruct path
                        let mut path = vec![to];
                        let mut node = to;
                        while let Some(&p) = parent.get(&node) {
                            path.push(p);
                            node = p;
                        }
                        path.reverse();
                        return Some(path);
                    }

                    queue.push_back(neighbor);
                }
            }
        }

        None
    }

    /// Get transformation between two frames at an epoch
    pub fn get_transform(&self, from: FrameId, to: FrameId, epoch: Epoch) -> Option<Se3> {
        if from == to {
            return Some(Se3::identity());
        }

        let path = self.find_path(from, to)?;

        // Compose transformations along path
        let mut result = Se3::identity();

        for window in path.windows(2) {
            let (f, t) = (window[0], window[1]);

            // Try forward edge
            if let Some(edge) = self.edges.get(&(f, t)) {
                let transform = (edge.transform)(epoch);
                result = result.compose(&transform);
            }
            // Try reverse edge (invert the transform)
            else if let Some(edge) = self.edges.get(&(t, f)) {
                let transform = (edge.transform)(epoch);
                result = result.compose(&transform.inverse());
            } else {
                return None;
            }
        }

        Some(result)
    }

    /// Transform a position vector between frames
    pub fn transform_position(
        &self,
        pos: DVec3,
        from: FrameId,
        to: FrameId,
        epoch: Epoch,
    ) -> Option<DVec3> {
        let se3 = self.get_transform(from, to, epoch)?;
        Some(se3.transform_point(pos))
    }

    /// Transform a conformal point between frames
    pub fn transform_conformal(
        &self,
        point: ConformalPoint,
        from: FrameId,
        to: FrameId,
        epoch: Epoch,
    ) -> Option<ConformalPoint> {
        let se3 = self.get_transform(from, to, epoch)?;
        let euclidean = point.to_euclidean();
        let transformed = se3.transform_point(euclidean);
        Some(ConformalPoint::from_euclidean(transformed))
    }

    /// Get transformation as a 4x4 matrix
    pub fn get_transform_matrix(&self, from: FrameId, to: FrameId, epoch: Epoch) -> Option<DMat4> {
        let se3 = self.get_transform(from, to, epoch)?;
        Some(se3.to_matrix())
    }

    /// Register built-in astronomical transformations
    fn register_builtin_transforms(&mut self) {
        // ICRF -> HCI: Translation from barycenter to Sun
        // (Simplified: assume Sun at barycenter for now)
        self.register_transform(FrameId::ICRF, FrameId::HCI, |_epoch| {
            // In reality, this would use ephemeris data
            Se3::identity()
        });

        // HCI -> HEE: Rotation so X points toward Earth
        self.register_transform(FrameId::HCI, FrameId::HEE, |epoch| {
            // Earth's longitude at epoch (simplified)
            let days = epoch.days_j2000();
            let earth_lon = (days / 365.25) * std::f64::consts::TAU;

            // Rotate so X points to Earth's current position
            Se3::from_rotation_z(-earth_lon)
        });

        // HCI -> GCI: Translation from Sun to Earth + obliquity rotation
        self.register_transform(FrameId::HCI, FrameId::GCI, |epoch| {
            let days = epoch.days_j2000();

            // Earth's position (simplified circular orbit at 1 AU)
            let earth_lon = (days / 365.25) * std::f64::consts::TAU;
            let earth_x = earth_lon.cos(); // AU
            let earth_y = earth_lon.sin();

            // Translation to Earth
            let translation = DVec3::new(-earth_x, -earth_y, 0.0);

            // Ecliptic obliquity (23.44 degrees)
            let obliquity = 23.44_f64.to_radians();

            // Compose: translate then rotate
            let t = Se3::from_translation(translation);
            let r = Se3::from_rotation_x(obliquity);
            t.compose(&r)
        });

        // GCI -> GSE: Rotation so X points toward Sun
        self.register_transform(FrameId::GCI, FrameId::GSE, |epoch| {
            let days = epoch.days_j2000();

            // Sun direction from Earth (opposite of Earth's position)
            let earth_lon = (days / 365.25) * std::f64::consts::TAU;
            let sun_dir = std::f64::consts::PI + earth_lon;

            // Rotate so X points to Sun
            Se3::from_rotation_z(-sun_dir)
        });
    }
}

impl Default for FrameGraph {
    fn default() -> Self {
        Self::with_builtins()
    }
}

// ─────────────────────────────────────────────────────────────────────────────────
// TESTS
// ─────────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_creation() {
        let graph = FrameGraph::with_builtins();
        assert!(graph.get_frame(FrameId::HCI).is_some());
        assert!(graph.get_frame(FrameId::GCI).is_some());
    }

    #[test]
    fn test_path_finding() {
        let graph = FrameGraph::with_builtins();

        // Direct path
        let path = graph.find_path(FrameId::HCI, FrameId::HEE);
        assert!(path.is_some());

        // Multi-hop path
        let path = graph.find_path(FrameId::ICRF, FrameId::GSE);
        assert!(path.is_some());
    }

    #[test]
    fn test_identity_transform() {
        let graph = FrameGraph::with_builtins();
        let epoch = Epoch::j2000();

        let transform = graph.get_transform(FrameId::HCI, FrameId::HCI, epoch);
        assert!(transform.is_some());

        let se3 = transform.unwrap();
        let p = DVec3::new(1.0, 2.0, 3.0);
        let result = se3.transform_point(p);
        assert!((result - p).length() < 1e-10);
    }

    #[test]
    fn test_hci_to_hee() {
        let graph = FrameGraph::with_builtins();
        let epoch = Epoch::j2000(); // At J2000, Earth is at specific position

        let transform = graph.get_transform(FrameId::HCI, FrameId::HEE, epoch);
        assert!(transform.is_some());
    }

    #[test]
    fn test_position_transform() {
        let graph = FrameGraph::with_builtins();
        let epoch = Epoch::j2000();

        let pos_hci = DVec3::new(1.0, 0.0, 0.0);
        let pos_gci = graph.transform_position(pos_hci, FrameId::HCI, FrameId::GCI, epoch);

        assert!(pos_gci.is_some());
    }

    #[test]
    fn test_user_frame_allocation() {
        let mut graph = FrameGraph::with_builtins();

        let id1 = graph.allocate_frame_id();
        let id2 = graph.allocate_frame_id();

        assert_ne!(id1, id2);
        assert!(id1.0 >= FrameId::USER_START);
    }
}
