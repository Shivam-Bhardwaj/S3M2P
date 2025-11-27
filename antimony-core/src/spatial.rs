use glam::Vec3;
use std::f32::consts::PI;
use std::collections::HashMap;
use std::sync::Arc;
use serde::{Serialize, Deserialize};

/// Represents a position in spherical coordinates (physics convention)
/// r: radial distance
/// theta: polar angle (0 to PI), angle from Z axis
/// phi: azimuthal angle (0 to 2PI), angle in XY plane from X axis
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SphericalPos {
    pub r: f32,
    pub theta: f32,
    pub phi: f32,
}

impl SphericalPos {
    pub fn new(r: f32, theta: f32, phi: f32) -> Self {
        Self { r, theta, phi }
    }

    pub fn from_cartesian(v: Vec3) -> Self {
        let r = v.length();
        if r < 1e-6 {
            return Self { r: 0.0, theta: 0.0, phi: 0.0 };
        }
        let theta = (v.z / r).acos();
        let phi = v.y.atan2(v.x);
        let phi = if phi < 0.0 { phi + 2.0 * PI } else { phi };
        Self { r, theta, phi }
    }

    pub fn to_cartesian(&self) -> Vec3 {
        let sin_theta = self.theta.sin();
        Vec3::new(
            self.r * sin_theta * self.phi.cos(),
            self.r * sin_theta * self.phi.sin(),
            self.r * self.theta.cos(),
        )
    }
}

/// A spatial index key representing a cell on the unit sphere surface.
/// Uses a Cube Sphere projection + Quadtree subdivision.
/// ID structure (64-bit):
/// | 3 bits (Face) | 5 bits (Level) | 28 bits (X) | 28 bits (Y) |
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SpatialKey(pub u64); // Pub so simple wrappers can access raw value

impl SpatialKey {
    const FACE_MASK: u64 = 0b111;
    const LEVEL_MASK: u64 = 0b11111;
    
    // Cube faces
    pub const FACE_POS_X: u8 = 0;
    pub const FACE_NEG_X: u8 = 1;
    pub const FACE_POS_Y: u8 = 2;
    pub const FACE_NEG_Y: u8 = 3;
    pub const FACE_POS_Z: u8 = 4;
    pub const FACE_NEG_Z: u8 = 5;

    pub fn new(face: u8, level: u8, x: u32, y: u32) -> Self {
        let f = (face as u64) & Self::FACE_MASK;
        let l = (level as u64) & Self::LEVEL_MASK;
        let x = (x as u64) & 0xFFFFFFF; // 28 bits
        let y = (y as u64) & 0xFFFFFFF; // 28 bits
        
        // Pack: Face(63-61) | Level(60-56) | X(55-28) | Y(27-0)
        let id = (f << 61) | (l << 56) | (x << 28) | y;
        Self(id)
    }

    pub fn from_point(p: Vec3, level: u8) -> Self {
        let abs = p.abs();
        let max_dim = if abs.x >= abs.y && abs.x >= abs.z {
            0 // X is max
        } else if abs.y >= abs.x && abs.y >= abs.z {
            1 // Y is max
        } else {
            2 // Z is max
        };

        let (face, u, v) = match max_dim {
            0 => if p.x >= 0.0 {
                (Self::FACE_POS_X, -p.z / p.x, -p.y / p.x)
            } else {
                (Self::FACE_NEG_X, -p.z / p.x, -p.y / p.x)
            },
            1 => if p.y >= 0.0 {
                (Self::FACE_POS_Y, p.x / p.y, p.z / p.y)
            } else {
                (Self::FACE_NEG_Y, p.x / p.y, p.z / p.y)
            },
            2 => if p.z >= 0.0 {
                (Self::FACE_POS_Z, p.x / p.z, -p.y / p.z)
            } else {
                (Self::FACE_NEG_Z, p.x / p.z, -p.y / p.z)
            },
            _ => unreachable!(),
        };

        // u, v are in range [-1, 1]. Map to [0, 1]
        let u_norm = (u + 1.0) * 0.5;
        let v_norm = (v + 1.0) * 0.5;

        // Map to integer grid at 2^level
        let dim = 1u32 << level;
        let x_idx = (u_norm * dim as f32).clamp(0.0, (dim - 1) as f32) as u32;
        let y_idx = (v_norm * dim as f32).clamp(0.0, (dim - 1) as f32) as u32;

        Self::new(face, level, x_idx, y_idx)
    }

    pub fn face(&self) -> u8 {
        ((self.0 >> 61) & Self::FACE_MASK) as u8
    }

    pub fn level(&self) -> u8 {
        ((self.0 >> 56) & Self::LEVEL_MASK) as u8
    }

    pub fn coords(&self) -> (u32, u32) {
        let x = (self.0 >> 28) & 0xFFFFFFF;
        let y = self.0 & 0xFFFFFFF;
        (x as u32, y as u32)
    }
    
    pub fn parent(&self) -> Option<Self> {
        let l = self.level();
        if l == 0 { return None; }
        
        let (x, y) = self.coords();
        Some(Self::new(self.face(), l - 1, x >> 1, y >> 1))
    }

    pub fn children(&self) -> [Self; 4] {
        let l = self.level();
        let (x, y) = self.coords();
        let next_l = l + 1;
        let next_x = x << 1;
        let next_y = y << 1;
        let f = self.face();
        
        [
            Self::new(f, next_l, next_x, next_y),
            Self::new(f, next_l, next_x + 1, next_y),
            Self::new(f, next_l, next_x, next_y + 1),
            Self::new(f, next_l, next_x + 1, next_y + 1),
        ]
    }

    /// Returns the approximate center direction of this cell
    pub fn direction(&self) -> Vec3 {
        let (x, y) = self.coords();
        let level = self.level();
        let dim = 1u32 << level;
        
        // Map integer coords back to [-1, 1]
        // Add 0.5 to get center of cell
        let u = ((x as f32 + 0.5) / dim as f32) * 2.0 - 1.0;
        let v = ((y as f32 + 0.5) / dim as f32) * 2.0 - 1.0;
        
        match self.face() {
            Self::FACE_POS_X => Vec3::new(1.0, -v, -u).normalize(),
            Self::FACE_NEG_X => Vec3::new(-1.0, -v, u).normalize(),
            Self::FACE_POS_Y => Vec3::new(u, 1.0, v).normalize(),
            Self::FACE_NEG_Y => Vec3::new(u, -1.0, -v).normalize(), // Check sign
            Self::FACE_POS_Z => Vec3::new(u, -v, 1.0).normalize(),
            Self::FACE_NEG_Z => Vec3::new(u, -v, -1.0).normalize(), // Check sign
            _ => Vec3::Y, // Fallback
        }
    }
}

/// Types of data layers for multi-resolution rendering
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DataLayer {
    Stars,
    Planets,
    Asteroids,
    Plasma,
    MagneticField,
}

/// State of a data chunk in the cache
#[derive(Clone, Debug)]
pub enum ChunkState<T> {
    Missing,
    Pending, // Fetch in progress
    Loaded(Vec<T>),
    Evicted,
}

/// Abstract backend for fetching spatial data
/// This allows swapping between local mock data, HTTP fetch, or WebSocket stream
pub trait SpatialBackend<T> {
    // For now, using synchronous return. In a real implementation, this would return a Future
    // or be part of an async actor system.
    fn fetch(&self, key: SpatialKey, layer: DataLayer) -> ChunkState<T>;
}

/// A container for spatially indexed data with Level of Detail (LOD)
/// and automatic retrieval support.
pub struct SpatialStore<T> {
    // Stores data chunks keyed by SpatialKey
    chunks: HashMap<SpatialKey, ChunkState<T>>,
    // Maximum depth of the tree
    max_level: u8,
    // Backend for fetching missing data
    backend: Option<Arc<dyn SpatialBackend<T> + Send + Sync>>,
}

impl<T> Default for SpatialStore<T> {
    fn default() -> Self {
        Self {
            chunks: HashMap::new(),
            max_level: 20,
            backend: None,
        }
    }
}

impl<T> SpatialStore<T> where T: Clone {
    pub fn new(max_level: u8) -> Self {
        Self {
            chunks: HashMap::new(),
            max_level,
            backend: None,
        }
    }

    pub fn with_backend(mut self, backend: Arc<dyn SpatialBackend<T> + Send + Sync>) -> Self {
        self.backend = Some(backend);
        self
    }

    pub fn insert(&mut self, key: SpatialKey, items: Vec<T>) {
        self.chunks.insert(key, ChunkState::Loaded(items));
    }

    pub fn get(&self, key: &SpatialKey) -> Option<&Vec<T>> {
        match self.chunks.get(key) {
            Some(ChunkState::Loaded(data)) => Some(data),
            _ => None,
        }
    }

    /// Query logic for Frustum/Field of View
    /// Returns keys that are visible. If they are missing, it triggers a fetch (conceptually).
    /// Returns: (Visible & Loaded Keys, Missing Keys to Fetch)
    pub fn query_visible_keys(&mut self, view_pos: Vec3, view_dir: Vec3, fov_rad: f32, detail_bias: f32, layer: DataLayer) -> (Vec<SpatialKey>, Vec<SpatialKey>) {
        let mut loaded_keys = Vec::new();
        let mut missing_keys = Vec::new();
        let mut stack = Vec::new();
        
        // Start with 6 root faces
        for face in 0..6 {
            stack.push(SpatialKey::new(face, 0, 0, 0));
        }

        while let Some(key) = stack.pop() {
            if !self.is_visible(key, view_pos, view_dir, fov_rad) {
                continue;
            }

            let desired_level = self.calculate_desired_level(key, view_pos, detail_bias);
            
            // If we are at the desired level or max level, check status
            if key.level() >= desired_level || key.level() >= self.max_level {
                match self.chunks.get(&key) {
                    Some(ChunkState::Loaded(_)) => loaded_keys.push(key),
                    Some(ChunkState::Pending) => {}, // Already fetching
                    _ => {
                        // Missing or Evicted
                        missing_keys.push(key);
                        if let Some(backend) = &self.backend {
                             // Trigger fetch
                             let _ = backend.fetch(key, layer);
                             // For now we just mark as Pending to avoid re-queueing immediately
                             // In a real system, the backend.fetch would handle the async dispatch
                             self.chunks.insert(key, ChunkState::Pending);
                        }
                    }
                }
            } else {
                // Need more detail, descend
                let children = key.children();
                for child in children {
                    stack.push(child);
                }
            }
        }
        
        (loaded_keys, missing_keys)
    }

    fn is_visible(&self, key: SpatialKey, _view_pos: Vec3, view_dir: Vec3, fov_rad: f32) -> bool {
        // Simple cone test
        let cell_dir = key.direction();
        let dot = view_dir.dot(cell_dir);
        // Angle between view_dir and cell_dir
        let angle = dot.acos();
        
        // Approximate cell angular size based on level
        // Level 0 covers ~90 deg, Level 1 ~45 deg, etc.
        let cell_angular_radius = (PI / 2.0) / (1u32 << key.level()) as f32;
        
        // Check if cone overlaps with cell
        angle - cell_angular_radius < fov_rad / 2.0
    }

    fn calculate_desired_level(&self, _key: SpatialKey, _view_pos: Vec3, detail_bias: f32) -> u8 {
        // Basic LOD heuristic:
        // Higher detail_bias -> Higher resolution (deeper level)
        // Real implementation would project the cell bounding box to screen space.
        // For now, we just use the bias.
        
        // E.g. if bias is 1.0, we go to level 3. If 2.0, level 5.
        let base_level = 3;
        (base_level as f32 * detail_bias) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spherical_cartesian_roundtrip() {
        let original = SphericalPos::new(10.0, PI / 4.0, PI / 3.0);
        let cart = original.to_cartesian();
        let roundtrip = SphericalPos::from_cartesian(cart);
        
        assert!((original.r - roundtrip.r).abs() < 1e-5);
        assert!((original.theta - roundtrip.theta).abs() < 1e-5);
        assert!((original.phi - roundtrip.phi).abs() < 1e-5);
    }

    #[test]
    fn test_spatial_key_mapping() {
        // Point on +Z axis
        let p = Vec3::new(0.0, 0.0, 1.0);
        let key = SpatialKey::from_point(p, 2);
        
        assert_eq!(key.face(), SpatialKey::FACE_POS_Z);
        assert_eq!(key.level(), 2);
        
        let (x, y) = key.coords();
        assert_eq!(x, 2);
        assert_eq!(y, 2);
    }
    
    #[test]
    fn test_spatial_key_hierarchy() {
        let p = Vec3::new(0.5, 0.5, 0.5).normalize();
        let level3 = SpatialKey::from_point(p, 3);
        let level2 = SpatialKey::from_point(p, 2);
        
        let parent = level3.parent().unwrap();
        assert_eq!(parent, level2, "Parent of Level 3 key should match Level 2 key for same point");
    }

    #[test]
    fn test_children_generation() {
        let root = SpatialKey::new(0, 0, 0, 0);
        let children = root.children();
        assert_eq!(children.len(), 4);
        for child in children {
            assert_eq!(child.level(), 1);
            assert_eq!(child.face(), 0);
            assert_eq!(child.parent(), Some(root));
        }
    }

    #[test]
    fn test_store_query() {
        let mut store = SpatialStore::<i32>::new(5);
        // Insert some data
        let key = SpatialKey::from_point(Vec3::X, 3);
        store.insert(key, vec![42]);
        
        let view_pos = Vec3::ZERO;
        let view_dir = Vec3::X;
        
        // Should see the key in front
        let (keys, missing) = store.query_visible_keys(view_pos, view_dir, PI / 2.0, 1.0, DataLayer::Stars);
        
        // The key we inserted is at Level 3.
        // Our simple heuristic with bias 1.0 wants Level 3.
        // So we should find it.
        
        assert!(keys.contains(&key), "Expected to find key {:?} in loaded keys", key);
        // In a sparse world, we expect many missing keys for the empty space we haven't populated
        assert!(!missing.contains(&key), "The loaded key should not be reported as missing");
        assert!(missing.len() > 0, "Should report missing keys for the rest of the visible area");
    }
}
