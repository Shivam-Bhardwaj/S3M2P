// AutoCrate Types

use super::constants::LumberSize;
use super::geometry::*;
use serde::{Deserialize, Serialize};

/// Product dimensions input (in inches)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProductDimensions {
    pub length: f32,
    pub width: f32,
    pub height: f32,
    pub weight: f32,
}

impl Default for ProductDimensions {
    fn default() -> Self {
        Self {
            length: 120.0,
            width: 120.0,
            height: 120.0,
            weight: 10000.0,
        }
    }
}

/// Clearances around product (in inches)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Clearances {
    pub side: f32,
    pub end: f32,
    pub top: f32,
}

impl Default for Clearances {
    fn default() -> Self {
        Self {
            side: 2.0,
            end: 2.0,
            top: 3.0,
        }
    }
}

/// Complete crate specification
#[derive(Clone, Debug)]
pub struct CrateSpec {
    pub product: ProductDimensions,
    pub clearances: Clearances,
    pub skid_count: u8,
    pub skid_size: LumberSize,
    pub floorboard_size: LumberSize,
    pub cleat_size: LumberSize,
}

impl Default for CrateSpec {
    fn default() -> Self {
        Self {
            product: ProductDimensions::default(),
            clearances: Clearances::default(),
            skid_count: 3,
            skid_size: LumberSize::L4x4,
            floorboard_size: LumberSize::L2x6,
            cleat_size: LumberSize::L1x4,
        }
    }
}

/// Generated crate geometry
#[derive(Clone, Debug)]
pub struct CrateGeometry {
    pub overall_length: f32,
    pub overall_width: f32,
    pub overall_height: f32,
    pub base_height: f32,
    pub skids: Vec<SkidGeometry>,
    pub floorboards: Vec<BoardGeometry>,
    pub panels: PanelSet,
    pub cleats: Vec<CleatGeometry>,
}
