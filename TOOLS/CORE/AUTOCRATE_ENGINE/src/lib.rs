//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: lib.rs | TOOLS/CORE/AUTOCRATE_ENGINE/src/lib.rs
//! PURPOSE: ASTM-standard shipping crate design automation engine
//! MODIFIED: 2025-12-09
//! LAYER: CORE → AUTOCRATE_ENGINE
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! AUTOCRATE_ENGINE generates parametric crate designs from product dimensions:
//! - Skid layout and sizing
//! - Floorboard placement
//! - Panel geometry (front/back/left/right/top)
//! - Cleat positioning
//! - Lumber bill of materials
//!
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │ ARCHITECTURE                                                                │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │   AutoCrateEngine                                                           │
//! │       │                                                                     │
//! │       ├── CrateSpec           (DNA/autocrate/types)                         │
//! │       ├── CrateGeometry       (DNA/autocrate/types)                         │
//! │       ├── LumberSize          (DNA/autocrate/constants)                     │
//! │       └── calculate_crate()   (DNA/autocrate/calculator)                    │
//! │                                                                             │
//! │   Design flow:                                                              │
//! │   1. Specify product dimensions (L x W x H, weight)                         │
//! │   2. Set clearances (side, end, top)                                        │
//! │   3. Select lumber sizes (skid, floorboard, cleat)                          │
//! │   4. Calculate complete crate geometry                                      │
//! │   5. Export to STEP/NX expressions                                          │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//!
//! DEPENDS ON:
//!   • DNA/autocrate → Crate types and algorithms
//!
//! USED BY:
//!   • TOOLS/AUTOCRATE → Interactive crate designer
//!
//! ═══════════════════════════════════════════════════════════════════════════════

// ─────────────────────────────────────────────────────────────────────────────────
// CODE BELOW - Optimized for ML development
// ─────────────────────────────────────────────────────────────────────────────────

// Re-export all autocrate types from DNA
pub use dna::autocrate::{
    // Calculator
    calculate_crate,
    BoardGeometry,
    BoundingBox,
    Clearances,
    CleatGeometry,
    CrateDesign,
    CrateGeometry,
    CratePart,
    CratePartKind,
    CrateSpec,
    KlimpPosition,
    LagScrewPosition,
    PartCategory,
    PartMaterial,
    // Constants
    LumberSize,
    PanelGeometry,
    PanelSet,
    PanelStopGeometry,
    PanelType,
    // Geometry
    Point3,
    // Types
    ProductDimensions,
    SkidGeometry,
};

// Re-export report generation (BOM + Cut List CSV)
pub use dna::autocrate::reports::{
    bom_to_csv, cut_list_to_csv, generate_bom, generate_cut_list, BomRow, CutListRow,
};

// Re-export STEP export (NX-importable assembly, inches)
pub use dna::export::step::{export_step_ap242, StepExportOptions};

/// Quick crate design with standard defaults
///
/// Takes just the product dimensions and weight, uses standard clearances
/// and lumber sizes suitable for most applications.
pub fn quick_design(length: f32, width: f32, height: f32, weight: f32) -> CrateGeometry {
    let spec = CrateSpec {
        product: ProductDimensions {
            length,
            width,
            height,
            weight,
        },
        ..CrateSpec::default()
    };

    calculate_crate(&spec)
}

/// Build the canonical `CrateDesign` (parts graph) from a spec.
pub fn design_from_spec(spec: &CrateSpec) -> CrateDesign {
    CrateDesign::from_spec(spec)
}

/// Export STEP (Part-21) assembly for the given design.
pub fn export_step(design: &CrateDesign) -> String {
    export_step_ap242(design, &StepExportOptions::default())
}

/// Export BOM as CSV for the given design.
pub fn export_bom_csv(design: &CrateDesign) -> String {
    let bom = generate_bom(design);
    bom_to_csv(&bom)
}

/// Export cut list as CSV for the given design.
pub fn export_cut_list_csv(design: &CrateDesign) -> String {
    let cut = generate_cut_list(design);
    cut_list_to_csv(&cut)
}

/// Design a heavy-duty crate for weights over 5000 lbs
///
/// Uses larger lumber (4x6 skids, 2x8 floorboards) and 5 skids for heavy loads.
pub fn heavy_duty_design(length: f32, width: f32, height: f32, weight: f32) -> CrateGeometry {
    let spec = CrateSpec {
        product: ProductDimensions {
            length,
            width,
            height,
            weight,
        },
        clearances: Clearances {
            side: 3.0,
            end: 3.0,
            top: 4.0,
        },
        skid_count: 5,
        skid_size: LumberSize::L4x6,
        floorboard_size: LumberSize::L2x8,
        cleat_size: LumberSize::L2x4,
        ..CrateSpec::default()
    };

    calculate_crate(&spec)
}

/// Calculate total wood board feet for the crate
pub fn calculate_board_feet(geometry: &CrateGeometry) -> f32 {
    let mut total_bf = 0.0;

    // Skids
    for skid in &geometry.skids {
        let size = skid.bounds.size();
        let bf = (size.x * size.y * size.z) / 144.0; // Convert to board feet
        total_bf += bf;
    }

    // Floorboards
    for board in &geometry.floorboards {
        let size = board.bounds.size();
        let bf = (size.x * size.y * size.z) / 144.0;
        total_bf += bf;
    }

    // Cleats
    for cleat in &geometry.cleats {
        let size = cleat.bounds.size();
        let bf = (size.x * size.y * size.z) / 144.0;
        total_bf += bf;
    }

    total_bf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quick_design() {
        let geometry = quick_design(48.0, 36.0, 24.0, 500.0);

        // Should have positive dimensions
        assert!(geometry.overall_length > 48.0);
        assert!(geometry.overall_width > 36.0);
        assert!(geometry.overall_height > 24.0);
    }

    #[test]
    fn test_heavy_duty_design() {
        let geometry = heavy_duty_design(96.0, 72.0, 48.0, 8000.0);

        // Heavy duty should have 5 skids
        assert_eq!(geometry.skids.len(), 5);
    }

    #[test]
    fn test_board_feet_calculation() {
        let geometry = quick_design(48.0, 36.0, 24.0, 500.0);
        let bf = calculate_board_feet(&geometry);

        // Should have some board feet
        assert!(bf > 0.0);
    }

    #[test]
    fn test_design_and_exports_are_non_empty() {
        let spec = CrateSpec::default();
        let design = design_from_spec(&spec);
        assert!(!design.parts.is_empty());

        let bom = export_bom_csv(&design);
        assert!(bom.contains("item,size,quantity"));

        let cut = export_cut_list_csv(&design);
        assert!(cut.contains("item,material,nominal"));

        let step = export_step(&design);
        assert!(step.contains("ISO-10303-21;"));
        assert!(step.contains("CONVERSION_BASED_UNIT('INCH'"));
    }
}
