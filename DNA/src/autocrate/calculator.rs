//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: calculator.rs | DNA/src/autocrate/calculator.rs
//! PURPOSE: Shipping crate geometry calculator for dimensions and components
//! MODIFIED: 2025-12-09
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════

// Crate geometry calculator
// Computes all dimensions and component positions
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(clippy::let_and_return)]

use super::geometry::*;
use super::types::{CrateGeometry, CrateSpec};

/// Calculate complete crate geometry from specification
pub fn calculate_crate(spec: &CrateSpec) -> CrateGeometry {
    let product = &spec.product;
    let clearances = &spec.clearances;

    // Step 1: Calculate base dimensions
    let skid_dims = spec.skid_size.actual();
    let floorboard_dims = spec.floorboard_size.actual();

    let base_height = skid_dims.0 + floorboard_dims.0;

    // Step 2: Calculate overall dimensions
    let panel_thickness = super::constants::geometry::DEFAULT_PANEL_THICKNESS;

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
    let board_count =
        ((length - super::constants::geometry::STANDARD_TOLERANCE) / board_dims.1).floor() as usize;
    let total_board_length = board_count as f32 * board_dims.1;
    let start_y = -total_board_length / 2.0;

    let mut boards = Vec::with_capacity(board_count);

    for i in 0..board_count {
        let y = start_y + board_dims.1 * (i as f32 + 0.5);

        let min = Point3::new(-width / 2.0, y - board_dims.1 / 2.0, skid_height);
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
    let ground_clearance = super::constants::geometry::SIDE_PANEL_GROUND_CLEARANCE;
    let panel_height = height - base_height - ground_clearance;

    // Front panel (negative Y)
    let front = calculate_single_panel(
        PanelType::Front,
        width,
        panel_height,
        panel_thickness,
        Point3::new(
            0.0,
            -length / 2.0 + panel_thickness / 2.0,
            base_height + ground_clearance,
        ),
        spec,
    );

    // Back panel (positive Y)
    let back = calculate_single_panel(
        PanelType::Back,
        width,
        panel_height,
        panel_thickness,
        Point3::new(
            0.0,
            length / 2.0 - panel_thickness / 2.0,
            base_height + ground_clearance,
        ),
        spec,
    );

    // Left panel (negative X)
    let left = calculate_single_panel(
        PanelType::Left,
        length - 2.0 * panel_thickness, // Between front and back
        panel_height,
        panel_thickness,
        Point3::new(
            -width / 2.0 + panel_thickness / 2.0,
            0.0,
            base_height + ground_clearance,
        ),
        spec,
    );

    // Right panel (positive X)
    let right = calculate_single_panel(
        PanelType::Right,
        length - 2.0 * panel_thickness,
        panel_height,
        panel_thickness,
        Point3::new(
            width / 2.0 - panel_thickness / 2.0,
            0.0,
            base_height + ground_clearance,
        ),
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

    PanelSet {
        front,
        back,
        left,
        right,
        top,
    }
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
            Point3::new(
                center.x + dim1 / 2.0,
                center.y + thickness / 2.0,
                center.z + dim2,
            ),
        ),
        PanelType::Left | PanelType::Right => BoundingBox::new(
            Point3::new(center.x - thickness / 2.0, center.y - dim1 / 2.0, center.z),
            Point3::new(
                center.x + thickness / 2.0,
                center.y + dim1 / 2.0,
                center.z + dim2,
            ),
        ),
        PanelType::Top => BoundingBox::new(
            Point3::new(
                center.x - dim1 / 2.0,
                center.y - dim2 / 2.0,
                center.z - thickness / 2.0,
            ),
            Point3::new(
                center.x + dim1 / 2.0,
                center.y + dim2 / 2.0,
                center.z + thickness / 2.0,
            ),
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
    let (cleat_thickness, cleat_width) = spec.cleat_size.actual(); // (height/thickness, width)
    let panel_thickness = spec.materials.panel_thickness;

    // Reconstruct panel bounds using the same logic as `calculate_single_panel`.
    // (We want cleats positioned deterministically relative to panel geometry.)
    let bounds = match panel_type {
        PanelType::Front | PanelType::Back => BoundingBox::new(
            Point3::new(center.x - panel_width / 2.0, center.y - panel_thickness / 2.0, center.z),
            Point3::new(
                center.x + panel_width / 2.0,
                center.y + panel_thickness / 2.0,
                center.z + panel_height,
            ),
        ),
        PanelType::Left | PanelType::Right => BoundingBox::new(
            Point3::new(center.x - panel_thickness / 2.0, center.y - panel_width / 2.0, center.z),
            Point3::new(
                center.x + panel_thickness / 2.0,
                center.y + panel_width / 2.0,
                center.z + panel_height,
            ),
        ),
        PanelType::Top => BoundingBox::new(
            Point3::new(
                center.x - panel_width / 2.0,
                center.y - panel_height / 2.0,
                center.z - panel_thickness / 2.0,
            ),
            Point3::new(
                center.x + panel_width / 2.0,
                center.y + panel_height / 2.0,
                center.z + panel_thickness / 2.0,
            ),
        ),
    };

    let mut cleats: Vec<CleatGeometry> = Vec::new();

    // Helper: compute evenly spaced centers between two endpoints (inclusive endpoints handled by caller).
    fn intermediate_centers(start_center: f32, end_center: f32, max_spacing: f32) -> Vec<f32> {
        if max_spacing <= 0.0 {
            return Vec::new();
        }
        let span = (end_center - start_center).abs();
        if span <= max_spacing {
            return Vec::new();
        }
        let intervals = (span / max_spacing).ceil() as usize;
        if intervals <= 1 {
            return Vec::new();
        }
        let step = span / (intervals as f32);
        (1..intervals)
            .map(|i| start_center + (end_center - start_center).signum() * (step * i as f32))
            .collect()
    }

    let max_spacing = super::constants::cleat::MAX_VERTICAL_SPACING;

    // Cleat placement differs by panel orientation.
    match panel_type {
        // FRONT/BACK: panel plane is XZ, thickness axis is Y.
        PanelType::Front | PanelType::Back => {
            // Choose the "outer" side of the panel (where cleats live within the panel thickness stack).
            let (y0, y1) = if panel_type == PanelType::Front {
                (bounds.min.y, bounds.min.y + cleat_thickness)
            } else {
                (bounds.max.y - cleat_thickness, bounds.max.y)
            };

            let x_min = bounds.min.x;
            let x_max = bounds.max.x;
            let z_min = bounds.min.z;
            let z_max = bounds.max.z;

            // Perimeter: bottom & top horizontals.
            cleats.push(CleatGeometry {
                bounds: BoundingBox::new(
                    Point3::new(x_min, y0, z_min),
                    Point3::new(x_max, y1, (z_min + cleat_width).min(z_max)),
                ),
                lumber_size: spec.cleat_size,
                panel: panel_type,
                is_vertical: false,
            });
            cleats.push(CleatGeometry {
                bounds: BoundingBox::new(
                    Point3::new(x_min, y0, (z_max - cleat_width).max(z_min)),
                    Point3::new(x_max, y1, z_max),
                ),
                lumber_size: spec.cleat_size,
                panel: panel_type,
                is_vertical: false,
            });

            // Perimeter: left & right verticals (between horizontals).
            let vz0 = (z_min + cleat_width).min(z_max);
            let vz1 = (z_max - cleat_width).max(z_min);
            if vz1 > vz0 {
                cleats.push(CleatGeometry {
                    bounds: BoundingBox::new(
                        Point3::new(x_min, y0, vz0),
                        Point3::new((x_min + cleat_width).min(x_max), y1, vz1),
                    ),
                    lumber_size: spec.cleat_size,
                    panel: panel_type,
                    is_vertical: true,
                });
                cleats.push(CleatGeometry {
                    bounds: BoundingBox::new(
                        Point3::new((x_max - cleat_width).max(x_min), y0, vz0),
                        Point3::new(x_max, y1, vz1),
                    ),
                    lumber_size: spec.cleat_size,
                    panel: panel_type,
                    is_vertical: true,
                });
            }

            // Intermediate vertical cleats across width.
            let left_center = x_min + cleat_width / 2.0;
            let right_center = x_max - cleat_width / 2.0;
            for (i, c) in intermediate_centers(left_center, right_center, max_spacing)
                .into_iter()
                .enumerate()
            {
                let cx0 = (c - cleat_width / 2.0).max(x_min);
                let cx1 = (c + cleat_width / 2.0).min(x_max);
                if cx1 <= cx0 || vz1 <= vz0 {
                    continue;
                }
                cleats.push(CleatGeometry {
                    bounds: BoundingBox::new(Point3::new(cx0, y0, vz0), Point3::new(cx1, y1, vz1)),
                    lumber_size: spec.cleat_size,
                    panel: panel_type,
                    is_vertical: true,
                });

                // Keep deterministic ordering by index (i) even if we don't store it yet.
                let _ = i;
            }
        }

        // LEFT/RIGHT: panel plane is YZ, thickness axis is X.
        PanelType::Left | PanelType::Right => {
            let (x0, x1) = if panel_type == PanelType::Left {
                (bounds.min.x, bounds.min.x + cleat_thickness)
            } else {
                (bounds.max.x - cleat_thickness, bounds.max.x)
            };

            let y_min = bounds.min.y;
            let y_max = bounds.max.y;
            let z_min = bounds.min.z;
            let z_max = bounds.max.z;

            // Perimeter: vertical cleats at front/back edges (full height).
            cleats.push(CleatGeometry {
                bounds: BoundingBox::new(
                    Point3::new(x0, y_min, z_min),
                    Point3::new(x1, (y_min + cleat_width).min(y_max), z_max),
                ),
                lumber_size: spec.cleat_size,
                panel: panel_type,
                is_vertical: true,
            });
            cleats.push(CleatGeometry {
                bounds: BoundingBox::new(
                    Point3::new(x0, (y_max - cleat_width).max(y_min), z_min),
                    Point3::new(x1, y_max, z_max),
                ),
                lumber_size: spec.cleat_size,
                panel: panel_type,
                is_vertical: true,
            });

            // Perimeter: bottom/top horizontals between vertical edges.
            let hy0 = (y_min + cleat_width).min(y_max);
            let hy1 = (y_max - cleat_width).max(y_min);
            if hy1 > hy0 {
                cleats.push(CleatGeometry {
                    bounds: BoundingBox::new(
                        Point3::new(x0, hy0, z_min),
                        Point3::new(x1, hy1, (z_min + cleat_width).min(z_max)),
                    ),
                    lumber_size: spec.cleat_size,
                    panel: panel_type,
                    is_vertical: false,
                });
                cleats.push(CleatGeometry {
                    bounds: BoundingBox::new(
                        Point3::new(x0, hy0, (z_max - cleat_width).max(z_min)),
                        Point3::new(x1, hy1, z_max),
                    ),
                    lumber_size: spec.cleat_size,
                    panel: panel_type,
                    is_vertical: false,
                });
            }

            // Intermediate vertical cleats along Y, between horizontals.
            let z0 = (z_min + cleat_width).min(z_max);
            let z1 = (z_max - cleat_width).max(z_min);
            let front_center = y_min + cleat_width / 2.0;
            let back_center = y_max - cleat_width / 2.0;
            for c in intermediate_centers(front_center, back_center, max_spacing) {
                let cy0 = (c - cleat_width / 2.0).max(y_min);
                let cy1 = (c + cleat_width / 2.0).min(y_max);
                if cy1 <= cy0 || z1 <= z0 {
                    continue;
                }
                cleats.push(CleatGeometry {
                    bounds: BoundingBox::new(Point3::new(x0, cy0, z0), Point3::new(x1, cy1, z1)),
                    lumber_size: spec.cleat_size,
                    panel: panel_type,
                    is_vertical: true,
                });
            }
        }

        // TOP: panel plane is XY, thickness axis is Z. We place cleats on the underside (min.z side).
        PanelType::Top => {
            let z0 = bounds.min.z;
            let z1 = (bounds.min.z + cleat_thickness).min(bounds.max.z);

            let x_min = bounds.min.x;
            let x_max = bounds.max.x;
            let y_min = bounds.min.y;
            let y_max = bounds.max.y;

            // Perimeter: vertical cleats at left/right edges (full length).
            cleats.push(CleatGeometry {
                bounds: BoundingBox::new(
                    Point3::new(x_min, y_min, z0),
                    Point3::new((x_min + cleat_width).min(x_max), y_max, z1),
                ),
                lumber_size: spec.cleat_size,
                panel: panel_type,
                is_vertical: true,
            });
            cleats.push(CleatGeometry {
                bounds: BoundingBox::new(
                    Point3::new((x_max - cleat_width).max(x_min), y_min, z0),
                    Point3::new(x_max, y_max, z1),
                ),
                lumber_size: spec.cleat_size,
                panel: panel_type,
                is_vertical: true,
            });

            // Perimeter: horizontals at front/back edges between vertical cleats.
            let hx0 = (x_min + cleat_width).min(x_max);
            let hx1 = (x_max - cleat_width).max(x_min);
            if hx1 > hx0 {
                cleats.push(CleatGeometry {
                    bounds: BoundingBox::new(
                        Point3::new(hx0, y_min, z0),
                        Point3::new(hx1, (y_min + cleat_width).min(y_max), z1),
                    ),
                    lumber_size: spec.cleat_size,
                    panel: panel_type,
                    is_vertical: false,
                });
                cleats.push(CleatGeometry {
                    bounds: BoundingBox::new(
                        Point3::new(hx0, (y_max - cleat_width).max(y_min), z0),
                        Point3::new(hx1, y_max, z1),
                    ),
                    lumber_size: spec.cleat_size,
                    panel: panel_type,
                    is_vertical: false,
                });
            }

            // Intermediate vertical cleats along X (run full Y length).
            let left_center = x_min + cleat_width / 2.0;
            let right_center = x_max - cleat_width / 2.0;
            for c in intermediate_centers(left_center, right_center, max_spacing) {
                let cx0 = (c - cleat_width / 2.0).max(x_min);
                let cx1 = (c + cleat_width / 2.0).min(x_max);
                if cx1 <= cx0 {
                    continue;
                }
                cleats.push(CleatGeometry {
                    bounds: BoundingBox::new(Point3::new(cx0, y_min, z0), Point3::new(cx1, y_max, z1)),
                    lumber_size: spec.cleat_size,
                    panel: panel_type,
                    is_vertical: true,
                });
            }
        }
    }

    cleats
}

#[cfg(test)]
mod tests {
    use super::super::types::CrateSpec;
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
