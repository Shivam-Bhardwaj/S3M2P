//! Crate generation algorithms for Style A and Style B

use crate::{
    assembly::*, constants::*, geometry::*, CrateSpec, CrateStyle, ProductDimensions, Clearances,
};

/// Generate complete crate assembly
pub fn generate_crate(spec: &CrateSpec, style: CrateStyle) -> CrateAssembly {
    let mut assembly = CrateAssembly::new();

    // Calculate dimensions
    let skid_dims = spec.skid_size.actual();
    let floor_dims = spec.floorboard_size.actual();
    let cleat_dims = spec.cleat_size.actual();

    let base_height = skid_dims.0 + floor_dims.0;
    let panel_thickness = geometry::DEFAULT_PANEL_THICKNESS;

    let interior_width = spec.product.width + 2.0 * spec.clearances.side;
    let interior_length = spec.product.length + 2.0 * spec.clearances.end;
    let interior_height = spec.product.height + spec.clearances.top;

    let overall_width = interior_width + 2.0 * panel_thickness;
    let overall_length = interior_length + 2.0 * panel_thickness;
    let overall_height = interior_height + base_height + panel_thickness;

    // Generate base assembly
    generate_base(&mut assembly, spec, overall_width, overall_length, base_height);

    // Generate walls based on style
    match style {
        CrateStyle::A => {
            // Style A: Open frame only
            generate_open_walls(&mut assembly, spec, overall_width, overall_length, interior_height, base_height);
        }
        CrateStyle::B => {
            // Style B: Frame + sheathing
            generate_sheathed_walls(&mut assembly, spec, overall_width, overall_length, interior_height, base_height, panel_thickness);
        }
    }

    assembly
}

fn generate_base(
    assembly: &mut CrateAssembly,
    spec: &CrateSpec,
    overall_width: f32,
    overall_length: f32,
    base_height: f32,
) {
    let skid_dims = spec.skid_size.actual();
    let floor_dims = spec.floorboard_size.actual();

    let half_width = overall_width / 2.0;
    let half_length = overall_length / 2.0;

    // Calculate skid positions (evenly spaced)
    let skid_count = spec.skid_count as usize;
    let skid_spacing = overall_width / (skid_count as f32 + 1.0);

    // Create skids
    for i in 0..skid_count {
        let x_pos = -half_width + skid_spacing * (i as f32 + 1.0);
        let bounds = BoundingBox {
            min: Point3::new(x_pos - skid_dims.1 / 2.0, -half_length, 0.0),
            max: Point3::new(x_pos + skid_dims.1 / 2.0, half_length, skid_dims.0),
        };

        let id = assembly.create_node(
            format!("Skid{}", i),
            ComponentType::Skid {
                lumber: spec.skid_size,
                length: overall_length,
            },
            LocalTransform::from_translation(Point3::new(x_pos, 0.0, 0.0)),
            bounds,
        );
        assembly.add_child(assembly.root_id, id);
    }

    // Create floor boards (spanning across skids)
    let board_count = (overall_length / floor_dims.1).ceil() as usize;
    let board_spacing = overall_length / board_count as f32;

    for i in 0..board_count {
        let y_pos = -half_length + i as f32 * board_spacing;
        let bounds = BoundingBox {
            min: Point3::new(-half_width, y_pos, skid_dims.0),
            max: Point3::new(half_width, y_pos + floor_dims.1, base_height),
        };

        let id = assembly.create_node(
            format!("Floorboard{}", i),
            ComponentType::Floorboard {
                lumber: spec.floorboard_size,
                length: overall_width,
            },
            LocalTransform::from_translation(Point3::new(0.0, y_pos, skid_dims.0)),
            bounds,
        );
        assembly.add_child(assembly.root_id, id);
    }
}

fn generate_sheathed_walls(
    assembly: &mut CrateAssembly,
    spec: &CrateSpec,
    overall_width: f32,
    overall_length: f32,
    interior_height: f32,
    base_height: f32,
    panel_thickness: f32,
) {
    let cleat_dims = spec.cleat_size.actual();
    let half_width = overall_width / 2.0;
    let half_length = overall_length / 2.0;
    let wall_height = interior_height;

    // 4 corner posts
    let corners = [
        ("CornerFL", -half_width + cleat_dims.1 / 2.0, -half_length + cleat_dims.1 / 2.0),
        ("CornerFR", half_width - cleat_dims.1 / 2.0, -half_length + cleat_dims.1 / 2.0),
        ("CornerBL", -half_width + cleat_dims.1 / 2.0, half_length - cleat_dims.1 / 2.0),
        ("CornerBR", half_width - cleat_dims.1 / 2.0, half_length - cleat_dims.1 / 2.0),
    ];

    for (name, x, y) in corners.iter() {
        let bounds = BoundingBox {
            min: Point3::new(x - cleat_dims.0 / 2.0, y - cleat_dims.1 / 2.0, base_height),
            max: Point3::new(x + cleat_dims.0 / 2.0, y + cleat_dims.1 / 2.0, base_height + wall_height),
        };

        let id = assembly.create_node(
            name.to_string(),
            ComponentType::Cleat {
                lumber: spec.cleat_size,
                length: wall_height,
                is_vertical: true,
            },
            LocalTransform::from_translation(Point3::new(*x, *y, base_height)),
            bounds,
        );
        assembly.add_child(assembly.root_id, id);
    }

    // Panels
    let panels = [
        (PanelType::Front, 0.0, -half_length - panel_thickness / 2.0, overall_width, wall_height),
        (PanelType::Back, 0.0, half_length + panel_thickness / 2.0, overall_width, wall_height),
        (PanelType::Left, -half_width - panel_thickness / 2.0, 0.0, overall_length, wall_height),
        (PanelType::Right, half_width + panel_thickness / 2.0, 0.0, overall_length, wall_height),
        (PanelType::Top, 0.0, 0.0, overall_width, overall_length),
    ];

    for (panel_type, x, y, width, height) in panels.iter() {
        let (min, max) = match panel_type {
            PanelType::Front | PanelType::Back => {
                let z = if *panel_type == PanelType::Top {
                    base_height + wall_height
                } else {
                    base_height
                };
                (
                    Point3::new(-width / 2.0, *y - panel_thickness / 2.0, z),
                    Point3::new(width / 2.0, *y + panel_thickness / 2.0, z + height),
                )
            }
            PanelType::Left | PanelType::Right => {
                (
                    Point3::new(*x - panel_thickness / 2.0, -width / 2.0, base_height),
                    Point3::new(*x + panel_thickness / 2.0, width / 2.0, base_height + height),
                )
            }
            PanelType::Top => {
                (
                    Point3::new(-width / 2.0, -height / 2.0, base_height + wall_height),
                    Point3::new(width / 2.0, height / 2.0, base_height + wall_height + panel_thickness),
                )
            }
        };

        let bounds = BoundingBox { min, max };

        let id = assembly.create_node(
            format!("{:?}Panel", panel_type),
            ComponentType::Panel {
                thickness: panel_thickness,
                width: *width,
                height: *height,
                panel_type: *panel_type,
            },
            LocalTransform::from_translation(Point3::new(*x, *y, base_height)),
            bounds,
        );
        assembly.add_child(assembly.root_id, id);
    }
}

fn generate_open_walls(
    assembly: &mut CrateAssembly,
    spec: &CrateSpec,
    overall_width: f32,
    overall_length: f32,
    interior_height: f32,
    base_height: f32,
) {
    // Style A: just corner posts and rails, no panels
    let cleat_dims = spec.cleat_size.actual();
    let half_width = overall_width / 2.0;
    let half_length = overall_length / 2.0;

    // 4 corner posts
    let corners = [
        ("CornerFL", -half_width, -half_length),
        ("CornerFR", half_width, -half_length),
        ("CornerBL", -half_width, half_length),
        ("CornerBR", half_width, half_length),
    ];

    for (name, x, y) in corners.iter() {
        let bounds = BoundingBox {
            min: Point3::new(*x, *y, base_height),
            max: Point3::new(x + cleat_dims.0, y + cleat_dims.1, base_height + interior_height),
        };

        let id = assembly.create_node(
            name.to_string(),
            ComponentType::Cleat {
                lumber: spec.cleat_size,
                length: interior_height,
                is_vertical: true,
            },
            LocalTransform::from_translation(Point3::new(*x, *y, base_height)),
            bounds,
        );
        assembly.add_child(assembly.root_id, id);
    }
}
