// Geometry types for crate components

use super::constants::LumberSize;
use glam::Vec3;
use serde::{Deserialize, Serialize};

/// 3D point (origin-based coordinate system)
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Point3 {
    pub x: f32, // Width (left/right)
    pub y: f32, // Length (front/back)
    pub z: f32, // Height (up)
}

impl Point3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn to_vec3(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}

impl From<Vec3> for Point3 {
    fn from(v: Vec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

/// Axis-aligned bounding box
#[derive(Clone, Copy, Debug)]
pub struct BoundingBox {
    pub min: Point3,
    pub max: Point3,
}

impl BoundingBox {
    pub fn new(min: Point3, max: Point3) -> Self {
        Self { min, max }
    }

    pub fn size(&self) -> Point3 {
        Point3 {
            x: (self.max.x - self.min.x).abs(),
            y: (self.max.y - self.min.y).abs(),
            z: (self.max.z - self.min.z).abs(),
        }
    }

    pub fn center(&self) -> Point3 {
        Point3 {
            x: (self.min.x + self.max.x) / 2.0,
            y: (self.min.y + self.max.y) / 2.0,
            z: (self.min.z + self.max.z) / 2.0,
        }
    }
}

/// Skid geometry
#[derive(Clone, Debug)]
pub struct SkidGeometry {
    pub bounds: BoundingBox,
    pub lumber_size: LumberSize,
    pub index: usize,
}

/// Floorboard/board geometry
#[derive(Clone, Debug)]
pub struct BoardGeometry {
    pub bounds: BoundingBox,
    pub lumber_size: LumberSize,
    pub index: usize,
}

/// Cleat geometry
#[derive(Clone, Debug)]
pub struct CleatGeometry {
    pub bounds: BoundingBox,
    pub lumber_size: LumberSize,
    pub panel: PanelType,
    pub is_vertical: bool,
}

/// Panel types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelType {
    Front,
    Back,
    Left,
    Right,
    Top,
}

impl PanelType {
    pub fn name(&self) -> &'static str {
        match self {
            PanelType::Front => "Front",
            PanelType::Back => "Back",
            PanelType::Left => "Left",
            PanelType::Right => "Right",
            PanelType::Top => "Top",
        }
    }
}

/// Panel geometry
#[derive(Clone, Debug)]
pub struct PanelGeometry {
    pub bounds: BoundingBox,
    pub panel_type: PanelType,
    pub thickness: f32,
    pub cleats: Vec<CleatGeometry>,
}

/// Complete panel set
#[derive(Clone, Debug)]
pub struct PanelSet {
    pub front: PanelGeometry,
    pub back: PanelGeometry,
    pub left: PanelGeometry,
    pub right: PanelGeometry,
    pub top: PanelGeometry,
}

/// Klimp fastener position
#[derive(Clone, Debug)]
pub struct KlimpPosition {
    pub position: Point3,
    pub rotation: f32, // Radians around Z
}

/// Lag screw position
#[derive(Clone, Debug)]
pub struct LagScrewPosition {
    pub position: Point3,
    pub panel: PanelType,
}

/// Panel stop geometry
#[derive(Clone, Debug)]
pub struct PanelStopGeometry {
    pub bounds: BoundingBox,
    pub location: &'static str,
}
