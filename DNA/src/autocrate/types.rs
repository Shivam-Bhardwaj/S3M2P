//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: types.rs | DNA/src/autocrate/types.rs
//! PURPOSE: Defines ProductDimensions, Clearances, CrateSpec types
//! MODIFIED: 2025-12-09
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════

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
            length: 48.0,
            width: 36.0,
            height: 24.0,
            weight: 500.0,
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

/// Target standard profile for crate generation.
///
/// Note: We model standards as **parameterized profiles** (rules + limits), not as copied text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrateStandard {
    /// ASTM D6039-style wood crates (v1 scope).
    AstmD6039,
}

/// Shipping mode impacts compliance and marking requirements.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShippingMode {
    Domestic,
    Export,
}

/// ISPM-15 treatment method for wood packaging material (WPM).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ismp15Treatment {
    HeatTreated,
    MethylBromideFumigated,
}

/// Export compliance configuration (ISPM-15).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ismp15Config {
    /// Whether ISPM-15 compliance is required for this crate.
    pub required: bool,
    /// Treatment method when `required == true`.
    pub treatment: Option<Ismp15Treatment>,
    /// Optional mark text to include as a decal/label (freeform).
    pub mark_text: Option<String>,
}

impl Default for Ismp15Config {
    fn default() -> Self {
        Self {
            required: false,
            treatment: None,
            mark_text: None,
        }
    }
}

/// Wood member quality class (ASTM D6199-style concept).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WoodMemberClass {
    Class1,
    Class2,
    Class3,
}

/// Materials and availability constraints for generating the crate.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialSpec {
    /// Plywood thickness in inches (e.g., 0.25).
    pub plywood_thickness: f32,
    /// Total panel thickness in inches (plywood + cleats stack).
    pub panel_thickness: f32,
    /// Optional wood member quality class used for member selection/policy.
    pub wood_member_class: Option<WoodMemberClass>,
    /// Allow 3x4 skid lumber for lightweight crates (special-case availability).
    pub allow_3x4_lumber: bool,
    /// Optional allowed floorboard lumber sizes (restrict to what is available).
    pub available_floorboard_sizes: Option<Vec<LumberSize>>,
}

impl Default for MaterialSpec {
    fn default() -> Self {
        Self {
            plywood_thickness: super::constants::plywood::DEFAULT_THICKNESS,
            panel_thickness: super::constants::geometry::DEFAULT_PANEL_THICKNESS,
            wood_member_class: None,
            allow_3x4_lumber: false,
            available_floorboard_sizes: None,
        }
    }
}

/// Hardware placement settings.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HardwareSpec {
    /// Target spacing for lag screws (in inches). Typical range: 16–24.
    pub lag_screw_spacing: f32,
    /// Target spacing for klimps (in inches). Typical range: 16–24.
    pub klimp_target_spacing: f32,
}

impl Default for HardwareSpec {
    fn default() -> Self {
        Self {
            lag_screw_spacing: 21.0,
            klimp_target_spacing: 16.0,
        }
    }
}

/// Markings / decals configuration (visual + BOM items).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarkingsSpec {
    pub fragile_stencil: bool,
    pub handling_symbols: bool,
    pub autocrate_text: bool,
    /// Whether to include ISPM-15 marking as a decal when required.
    pub ispm15_mark: bool,
}

impl Default for MarkingsSpec {
    fn default() -> Self {
        Self {
            fragile_stencil: true,
            handling_symbols: true,
            autocrate_text: false,
            ispm15_mark: false,
        }
    }
}

/// Standard + shipping requirement profile.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequirementsSpec {
    pub standard: CrateStandard,
    pub shipping_mode: ShippingMode,
    pub ispm15: Ismp15Config,
}

impl Default for RequirementsSpec {
    fn default() -> Self {
        Self {
            standard: CrateStandard::AstmD6039,
            shipping_mode: ShippingMode::Domestic,
            ispm15: Ismp15Config::default(),
        }
    }
}

/// Complete crate specification
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrateSpec {
    pub product: ProductDimensions,
    pub clearances: Clearances,
    pub requirements: RequirementsSpec,
    pub materials: MaterialSpec,
    pub hardware: HardwareSpec,
    pub markings: MarkingsSpec,
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
            requirements: RequirementsSpec::default(),
            materials: MaterialSpec::default(),
            hardware: HardwareSpec::default(),
            markings: MarkingsSpec::default(),
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
