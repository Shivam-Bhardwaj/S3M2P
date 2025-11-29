// Crate geometry calculator
// Computes all dimensions and component positions

use crate::{CrateSpec, CrateGeometry, LumberSize};
use crate::constants;
use crate::geometry::*;

/// Calculate complete crate geometry from specification
pub fn calculate_crate(spec: &CrateSpec) -> CrateGeometry {
    let product = &spec.product;
    let clearances = &spec.clearances;

    // Step 1: Calculate base dimensions
    let skid_dims = spec.skid_size.actual();
    let floorboard_dims = spec.floorboard_size.actual();

    let base_height = skid_dims.0 + floorboard_dims.0;

    // Step 2: Calculate overall dimensions
    let panel_thickness = crate::constants::geometry::DEFAULT_PANEL_THICKNESS;

    let overall_width = product.width + 2.0 * clearances.side + 2.0 * panel_thickness;
    let overall_length = product.length + 2.0 * clearances.end + 2.0 * panel_thickness;
    let overall_height = product.height + clearances.top + base_height + panel_thickness;

    // Step 3: Calculate skids
    let skids = calculate_skids(spec, overall_length, overall_width);

    // Step 4: Calculate floorboards
    let floorboards = calculate_floorboards(spec, overall_length, overall_width, skid_dims.0);

    // Step 5: Calculate panels
    let panels = calculate_panels(
        overall_length,
        overall_width,
        overall_height,
        base_height,
        panel_thickness,
        spec,
    );

    // Step 6: Collect cleats from panels
    let mut cleats = Vec::new();
    cleats.extend(panels.front.cleats.iter().cloned());
    cleats.extend(panels.back.cleats.iter().cloned());
    cleats.extend(panels.left.cleats.iter().cloned());
    cleats.extend(panels.right.cleats.iter().cloned());
    cleats.extend(panels.top.cleats.iter().cloned());

    CrateGeometry {
        overall_length,
        overall_width,
        overall_height,
        base_height,
        skids,
        floorboards,
        panels,
        cleats,
    }
}

fn calculate_skids(spec: &CrateSpec, length: f32, width: f32) -> Vec<SkidGeometry> {
    let skid_dims = spec.skid_size.actual();
    let count = spec.skid_count as usize;

    let mut skids = Vec::with_capacity(count);

    // Skids run along length (Y axis)
    let skid_length = length;
    let total_skid_width = count as f32 * skid_dims.1;
    let gap = (width - total_skid_width) / (count + 1) as f32;

    for i in 0..count {
        let x_center = gap * (i + 1) as f32 + skid_dims.1 * (i as f32 + 0.5);

        let min = Point3::new(
            x_center - skid_dims.1 / 2.0 - width / 2.0,
            -skid_length / 2.0,
            0.0,
        );
        let max = Point3::new(
            x_center + skid_dims.1 / 2.0 - width / 2.0,
            skid_length / 2.0,
            skid_dims.0,
        );

        skids.push(SkidGeometry {
            bounds: BoundingBox::new(min, max),
            lumber_size: spec.skid_size,
            index: i,
        });
    }

    skids
}

fn calculate_floorboards(
    spec: &CrateSpec,
    length: f32,
    width: f32,
    skid_height: f32,
) -> Vec<BoardGeometry> {
    let board_dims = spec.floorboard_size.actual();

    // Floorboards run across width (X axis)
    let board_count = ((length - crate::constants::geometry::STANDARD_TOLERANCE) / board_dims.1).floor() as usize;
    let total_board_length = board_count as f32 * board_dims.1;
    let start_y = -total_board_length / 2.0;

    let mut boards = Vec::with_capacity(board_count);

    for i in 0..board_count {
        let y = start_y + board_dims.1 * (i as f32 + 0.5);

        let min = Point3::new(
            -width / 2.0,
            y - board_dims.1 / 2.0,
            skid_height,
        );
        let max = Point3::new(
            width / 2.0,
            y + board_dims.1 / 2.0,
            skid_height + board_dims.0,
        );

        boards.push(BoardGeometry {
            bounds: BoundingBox::new(min, max),
            lumber_size: spec.floorboard_size,
            index: i,
        });
    }

    boards
}

fn calculate_panels(
    length: f32,
    width: f32,
    height: f32,
    base_height: f32,
    panel_thickness: f32,
    spec: &CrateSpec,
) -> PanelSet {
    let ground_clearance = crate::constants::geometry::SIDE_PANEL_GROUND_CLEARANCE;
    let panel_height = height - base_height - ground_clearance;

    // Front panel (negative Y)
    let front = calculate_single_panel(
        PanelType::Front,
        width,
        panel_height,
        panel_thickness,
        Point3::new(0.0, -length / 2.0 + panel_thickness / 2.0, base_height + ground_clearance),
        spec,
    );

    // Back panel (positive Y)
    let back = calculate_single_panel(
        PanelType::Back,
        width,
        panel_height,
        panel_thickness,
        Point3::new(0.0, length / 2.0 - panel_thickness / 2.0, base_height + ground_clearance),
        spec,
    );

    // Left panel (negative X)
    let left = calculate_single_panel(
        PanelType::Left,
        length - 2.0 * panel_thickness, // Between front and back
        panel_height,
        panel_thickness,
        Point3::new(-width / 2.0 + panel_thickness / 2.0, 0.0, base_height + ground_clearance),
        spec,
    );

    // Right panel (positive X)
    let right = calculate_single_panel(
        PanelType::Right,
        length - 2.0 * panel_thickness,
        panel_height,
        panel_thickness,
        Point3::new(width / 2.0 - panel_thickness / 2.0, 0.0, base_height + ground_clearance),
        spec,
    );

    // Top panel
    let top = calculate_single_panel(
        PanelType::Top,
        width - 2.0 * panel_thickness,
        length - 2.0 * panel_thickness,
        panel_thickness,
        Point3::new(0.0, 0.0, height - panel_thickness / 2.0),
        spec,
    );

    PanelSet { front, back, left, right, top }
}

fn calculate_single_panel(
    panel_type: PanelType,
    dim1: f32, // Width or length
    dim2: f32, // Height or depth
    thickness: f32,
    center: Point3,
    spec: &CrateSpec,
) -> PanelGeometry {
    let bounds = match panel_type {
        PanelType::Front | PanelType::Back => BoundingBox::new(
            Point3::new(center.x - dim1 / 2.0, center.y - thickness / 2.0, center.z),
            Point3::new(center.x + dim1 / 2.0, center.y + thickness / 2.0, center.z + dim2),
        ),
        PanelType::Left | PanelType::Right => BoundingBox::new(
            Point3::new(center.x - thickness / 2.0, center.y - dim1 / 2.0, center.z),
            Point3::new(center.x + thickness / 2.0, center.y + dim1 / 2.0, center.z + dim2),
        ),
        PanelType::Top => BoundingBox::new(
            Point3::new(center.x - dim1 / 2.0, center.y - dim2 / 2.0, center.z - thickness / 2.0),
            Point3::new(center.x + dim1 / 2.0, center.y + dim2 / 2.0, center.z + thickness / 2.0),
        ),
    };

    // Calculate cleats for this panel
    let cleats = calculate_panel_cleats(panel_type, dim1, dim2, center, spec);

    PanelGeometry {
        bounds,
        panel_type,
        thickness,
        cleats,
    }
}

fn calculate_panel_cleats(
    panel_type: PanelType,
    panel_width: f32,
    panel_height: f32,
    center: Point3,
    spec: &CrateSpec,
) -> Vec<CleatGeometry> {
    let cleat_dims = spec.cleat_size.actual();
    let mut cleats = Vec::new();

    // Simplified cleat calculation - perimeter cleats only
    // Full implementation would include intermediate cleats based on MAX_VERTICAL_SPACING

    // For now, add corner/edge cleats
    // This is a placeholder - full implementation via issues

    cleats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_calculation() {
        let spec = CrateSpec::default();
        let geom = calculate_crate(&spec);

        assert!(geom.overall_width > spec.product.width);
        assert!(geom.overall_length > spec.product.length);
        assert!(geom.overall_height > spec.product.height);
    }

    #[test]
    fn test_skid_count() {
        let mut spec = CrateSpec::default();
        spec.skid_count = 4;
        let geom = calculate_crate(&spec);

        assert_eq!(geom.skids.len(), 4);
    }
}
