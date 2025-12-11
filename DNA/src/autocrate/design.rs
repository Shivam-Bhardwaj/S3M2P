//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: design.rs | DNA/src/autocrate/design.rs
//! PURPOSE: Canonical crate design graph (parts list) for export + visualization
//! MODIFIED: 2025-12-11
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════

use super::calculator::calculate_crate;
use super::constants::LumberSize;
use super::geometry::{BoundingBox, PanelType, Point3};
use super::types::{CrateGeometry, CrateSpec};

/// High-level part category (used by exporters/viewers to color/group items).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PartCategory {
    Lumber,
    Plywood,
    Hardware,
    Decal,
}

/// Specific part kind (semantic meaning within the crate).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CratePartKind {
    Skid { index: usize },
    Floorboard { index: usize },
    Panel { panel: PanelType },
    Cleat { panel: PanelType, is_vertical: bool, index: usize },
    PanelStop { location: String },
    Klimp { edge: String, index: usize },
    LagScrew { panel: PanelType, index: usize, component: String },
    Decal { panel: PanelType, kind: String, index: usize },
}

/// Material metadata for a part.
#[derive(Clone, Debug, PartialEq)]
pub enum PartMaterial {
    Lumber { nominal: LumberSize },
    Plywood { thickness_in: f32 },
    Hardware { sku: String },
    Decal { sku: String, text: Option<String> },
}

/// Canonical crate part record.
#[derive(Clone, Debug)]
pub struct CratePart {
    /// Stable identifier (deterministic for a given `CrateDesign`).
    pub id: String,
    /// Human-friendly name (used in UI/export labels).
    pub name: String,
    pub category: PartCategory,
    pub kind: CratePartKind,
    pub material: PartMaterial,
    /// Axis-aligned bounds in **global crate coordinates** (inches).
    pub bounds: BoundingBox,
    /// Optional freeform metadata (notes, standards tags, etc.).
    pub metadata: Option<String>,
}

/// Canonical design graph for downstream export + visualization.
///
/// This is the **single source of truth** for:
/// - STEP assembly export
/// - BOM + Cut List
/// - 3D viewer scene generation
#[derive(Clone, Debug)]
pub struct CrateDesign {
    pub spec: CrateSpec,
    pub geometry: CrateGeometry,
    pub parts: Vec<CratePart>,
}

impl CrateDesign {
    /// Build a canonical design from a spec by running the calculator.
    pub fn from_spec(spec: &CrateSpec) -> Self {
        let geometry = calculate_crate(spec);
        Self::from_geometry(spec.clone(), geometry)
    }

    /// Build a canonical design from an already computed geometry.
    pub fn from_geometry(spec: CrateSpec, geometry: CrateGeometry) -> Self {
        let mut parts: Vec<CratePart> = Vec::new();

        // Skids
        for (i, skid) in geometry.skids.iter().enumerate() {
            parts.push(CratePart {
                id: format!("SKID-{:02}", i + 1),
                name: format!("Skid {}", i + 1),
                category: PartCategory::Lumber,
                kind: CratePartKind::Skid { index: i },
                material: PartMaterial::Lumber {
                    nominal: skid.lumber_size,
                },
                bounds: skid.bounds,
                metadata: Some(format!("Lumber {}", skid.lumber_size.name())),
            });
        }

        // Floorboards
        for (i, board) in geometry.floorboards.iter().enumerate() {
            parts.push(CratePart {
                id: format!("FLOORBOARD-{:02}", i + 1),
                name: format!("Floorboard {}", i + 1),
                category: PartCategory::Lumber,
                kind: CratePartKind::Floorboard { index: i },
                material: PartMaterial::Lumber {
                    nominal: board.lumber_size,
                },
                bounds: board.bounds,
                metadata: Some(format!("Lumber {}", board.lumber_size.name())),
            });
        }

        // Panels (plywood + cleat stack represented as a single box for now)
        // Note: We keep the semantic panel identity (front/back/left/right/top).
        let panel_thickness = spec.materials.panel_thickness;
        for panel in [
            &geometry.panels.front,
            &geometry.panels.back,
            &geometry.panels.left,
            &geometry.panels.right,
            &geometry.panels.top,
        ] {
            let panel_name = panel.panel_type.name();
            parts.push(CratePart {
                id: format!("PANEL-{}", panel_name.to_uppercase()),
                name: format!("{} Panel", panel_name),
                category: PartCategory::Plywood,
                kind: CratePartKind::Panel {
                    panel: panel.panel_type,
                },
                material: PartMaterial::Plywood {
                    thickness_in: panel_thickness,
                },
                bounds: panel.bounds,
                metadata: Some(format!(
                    "Panel thickness {:.3} in (plywood {:.3} in)",
                    panel_thickness, spec.materials.plywood_thickness
                )),
            });
        }

        // Cleats (may be empty until cleat generation is implemented)
        // Grouped deterministically by (panel, orientation, index)
        let mut cleat_index: usize = 0;
        for cleat in geometry.cleats.iter() {
            let panel_name = cleat.panel.name();
            let orient = if cleat.is_vertical { "V" } else { "H" };
            parts.push(CratePart {
                id: format!("CLEAT-{}-{}-{:02}", panel_name.to_uppercase(), orient, cleat_index + 1),
                name: format!("{} {} Cleat {}", panel_name, orient, cleat_index + 1),
                category: PartCategory::Lumber,
                kind: CratePartKind::Cleat {
                    panel: cleat.panel,
                    is_vertical: cleat.is_vertical,
                    index: cleat_index,
                },
                material: PartMaterial::Lumber {
                    nominal: cleat.lumber_size,
                },
                bounds: cleat.bounds,
                metadata: Some(format!("Lumber {}", cleat.lumber_size.name())),
            });
            cleat_index += 1;
        }

        // ─────────────────────────────────────────────────────────────────────────
        // Hardware + decals (fasteners, panel stops, markings)
        // ─────────────────────────────────────────────────────────────────────────

        parts.extend(generate_panel_stops(&spec, &geometry));
        parts.extend(generate_klimps(&spec, &geometry));
        parts.extend(generate_lag_screws(&spec, &geometry));
        parts.extend(generate_decals(&spec, &geometry));

        Self {
            spec,
            geometry,
            parts,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers: Hardware + decals generation
// ─────────────────────────────────────────────────────────────────────────────

fn clamp_f32(v: f32, min: f32, max: f32) -> f32 {
    if v < min {
        min
    } else if v > max {
        max
    } else {
        v
    }
}

/// Generate evenly spaced positions including endpoints, respecting typical 16–24" spacing constraints.
fn spaced_positions(start: f32, end: f32, target_spacing: f32) -> Vec<f32> {
    let tolerance = 1e-4_f32;
    let min_spacing = 16.0_f32;
    let max_spacing = 24.0_f32;
    let target = clamp_f32(target_spacing, min_spacing, max_spacing);

    if end <= start + tolerance {
        return vec![(start + end) / 2.0];
    }

    let span = end - start;
    if span <= target + tolerance {
        return vec![start, end];
    }

    let mut intervals = (span / target).ceil() as usize;
    intervals = intervals.max(1);
    let mut spacing = span / intervals as f32;

    while intervals > 1 && spacing < min_spacing - tolerance {
        intervals -= 1;
        spacing = span / intervals as f32;
    }

    if intervals <= 1 {
        return vec![start, end];
    }

    (0..=intervals)
        .map(|i| start + (span * i as f32) / (intervals as f32))
        .collect()
}

fn generate_panel_stops(spec: &CrateSpec, geometry: &CrateGeometry) -> Vec<CratePart> {
    let thickness = super::constants::panel_stop::THICKNESS;
    let width = super::constants::panel_stop::WIDTH;
    let edge_inset = super::constants::panel_stop::EDGE_INSET;

    let front = &geometry.panels.front.bounds;
    let top = &geometry.panels.top.bounds;

    // Stop length = half of the smallest relevant panel edge.
    let front_w = front.size().x;
    let front_h = front.size().z;
    let side_l = geometry.panels.left.bounds.size().y;
    let side_h = geometry.panels.left.bounds.size().z;
    let top_w = top.size().x;
    let top_l = top.size().y;
    let smallest = front_w
        .min(front_h)
        .min(side_l)
        .min(side_h)
        .min(top_w)
        .min(top_l);
    let stop_length = smallest * 0.5;

    let mut parts = Vec::new();

    // Front panel stops: left/right edges, centered vertically, flush to inner face.
    let z_center = (front.min.z + front.max.z) / 2.0;
    let y0 = front.max.y; // inner face (front panel interior is +Y)
    let y1 = y0 + thickness;

    let x_left_center = front.min.x + edge_inset + width / 2.0;
    let x_right_center = front.max.x - edge_inset - width / 2.0;

    for (idx, (id, x_center)) in [
        ("PANEL_STOP-FRONT-LEFT", x_left_center),
        ("PANEL_STOP-FRONT-RIGHT", x_right_center),
    ]
    .into_iter()
    .enumerate()
    {
        let b = BoundingBox::new(
            Point3::new(x_center - width / 2.0, y0, z_center - stop_length / 2.0),
            Point3::new(x_center + width / 2.0, y1, z_center + stop_length / 2.0),
        );

        parts.push(CratePart {
            id: id.to_string(),
            name: format!("Front Panel Stop {}", idx + 1),
            category: PartCategory::Plywood,
            kind: CratePartKind::PanelStop {
                location: id.to_string(),
            },
            material: PartMaterial::Plywood {
                thickness_in: thickness,
            },
            bounds: b,
            metadata: Some(format!("3/8\" x 2\" plywood stop, length {:.2}\"", stop_length)),
        });
    }

    // Top panel front stop: underside near front edge.
    let top_z0 = top.min.z; // underside (top panel interior side)
    let top_z1 = top_z0 - thickness;
    let y_front = top.min.y + edge_inset;
    let b = BoundingBox::new(
        Point3::new(-stop_length / 2.0, y_front, top_z1),
        Point3::new(stop_length / 2.0, y_front + width, top_z0),
    );
    parts.push(CratePart {
        id: "PANEL_STOP-TOP-FRONT".to_string(),
        name: "Top Panel Stop (Front)".to_string(),
        category: PartCategory::Plywood,
        kind: CratePartKind::PanelStop {
            location: "PANEL_STOP-TOP-FRONT".to_string(),
        },
        material: PartMaterial::Plywood {
            thickness_in: thickness,
        },
        bounds: b,
        metadata: Some(format!("3/8\" x 2\" plywood stop, length {:.2}\"", stop_length)),
    });

    // Optional: if export requires ISPM-15, include the stop length note as part metadata only.
    let _ = spec;

    parts
}

fn generate_klimps(spec: &CrateSpec, geometry: &CrateGeometry) -> Vec<CratePart> {
    // Klimp geometry (inches) — based on the reference AutoCrate implementation.
    let longer = 4.0_f32;
    let shorter = 3.0_f32;
    let width = 1.0_f32;

    let target_spacing = spec.hardware.klimp_target_spacing;
    let sku = "KLIMP_FASTENER".to_string();

    let front = &geometry.panels.front.bounds;
    let cleat_width = spec.cleat_size.actual().1;

    // Outer face for front panel is at min.y (front is negative Y side).
    let y0 = front.min.y;

    let mut parts: Vec<CratePart> = Vec::new();

    // TOP EDGE: along X near panel top.
    let start_x = front.min.x + cleat_width + 1.0;
    let end_x = front.max.x - cleat_width - 1.0;
    if end_x > start_x {
        let positions = spaced_positions(start_x, end_x, target_spacing);
        for (i, x) in positions.into_iter().enumerate() {
            let b = BoundingBox::new(
                Point3::new(x - width / 2.0, y0, front.max.z - longer),
                Point3::new(x + width / 2.0, y0 + shorter, front.max.z),
            );
            parts.push(CratePart {
                id: format!("KLIMP-TOP-{:02}", i + 1),
                name: format!("Klimp (Top) {}", i + 1),
                category: PartCategory::Hardware,
                kind: CratePartKind::Klimp {
                    edge: "top".to_string(),
                    index: i,
                },
                material: PartMaterial::Hardware { sku: sku.clone() },
                bounds: b,
                metadata: Some("Front panel top-edge klimp".to_string()),
            });
        }
    }

    // SIDE EDGES: along Z at left/right X edges.
    let z_start = front.min.z + cleat_width + 2.0;
    let z_end = front.max.z - (cleat_width + 2.0);
    if z_end > z_start {
        let positions = spaced_positions(z_start, z_end, target_spacing);
        for (i, z) in positions.into_iter().enumerate() {
            for (edge, x_edge) in [("left", front.min.x), ("right", front.max.x)] {
                let b = BoundingBox::new(
                    Point3::new(x_edge - width / 2.0, y0, z - shorter / 2.0),
                    Point3::new(x_edge + width / 2.0, y0 + longer, z + shorter / 2.0),
                );
                parts.push(CratePart {
                    id: format!("KLIMP-{}-{:02}", edge.to_uppercase(), i + 1),
                    name: format!("Klimp ({}) {}", edge, i + 1),
                    category: PartCategory::Hardware,
                    kind: CratePartKind::Klimp {
                        edge: edge.to_string(),
                        index: i,
                    },
                    material: PartMaterial::Hardware { sku: sku.clone() },
                    bounds: b,
                    metadata: Some("Front panel side-edge klimp".to_string()),
                });
            }
        }
    }

    parts
}

fn generate_lag_screws(spec: &CrateSpec, geometry: &CrateGeometry) -> Vec<CratePart> {
    // Approximate vendor CAD dimensions (inches) from AutoCrate reference.
    let shank_length = 3.0_f32;
    let head_height = 0.25_f32;
    let head_diameter = 0.75_f32;
    let head_r = head_diameter / 2.0;

    let spacing = spec.hardware.lag_screw_spacing;
    let sku = "LAG_SCREW_0.38x3.00".to_string();

    let mut parts: Vec<CratePart> = Vec::new();

    // Helper: place along a panel, with axis-aligned bounding boxes.
    let mut add_for_panel = |panel: PanelType| {
        let (panel_bounds, axis) = match panel {
            PanelType::Left => (&geometry.panels.left.bounds, "X+"),
            PanelType::Right => (&geometry.panels.right.bounds, "X-"),
            PanelType::Front => (&geometry.panels.front.bounds, "Y+"),
            PanelType::Back => (&geometry.panels.back.bounds, "Y-"),
            _ => return,
        };

        // Choose outside face coordinate for the screw axis.
        let (axis_min, axis_max) = match panel {
            PanelType::Left => (panel_bounds.min.x - head_height, panel_bounds.min.x + shank_length),
            PanelType::Right => (panel_bounds.max.x - shank_length, panel_bounds.max.x + head_height),
            PanelType::Front => (panel_bounds.min.y - head_height, panel_bounds.min.y + shank_length),
            PanelType::Back => (panel_bounds.max.y - shank_length, panel_bounds.max.y + head_height),
            _ => (0.0, 0.0),
        };

        // Compute row positions based on vertical cleats when possible.
        let mut centers: Vec<f32> = geometry
            .cleats
            .iter()
            .filter(|c| c.panel == panel && c.is_vertical)
            .map(|c| match panel {
                PanelType::Left | PanelType::Right => c.bounds.center().y,
                PanelType::Front | PanelType::Back => c.bounds.center().x,
                _ => 0.0,
            })
            .collect();
        centers.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        centers.dedup_by(|a, b| (*a - *b).abs() < 1e-3);

        let (start, end) = if centers.len() >= 2 {
            (centers[0], centers[centers.len() - 1])
        } else {
            match panel {
                PanelType::Left | PanelType::Right => (
                    panel_bounds.min.y + head_r,
                    panel_bounds.max.y - head_r,
                ),
                PanelType::Front | PanelType::Back => (
                    panel_bounds.min.x + head_r,
                    panel_bounds.max.x - head_r,
                ),
                _ => (0.0, 0.0),
            }
        };

        let positions = spaced_positions(start, end, spacing);
        let z_center = panel_bounds.min.z + 1.0; // pragmatic: near base region

        for (i, pos) in positions.into_iter().enumerate() {
            let (x0, x1, y0, y1) = match panel {
                PanelType::Left | PanelType::Right => {
                    // Axis along X, row across Y
                    (axis_min, axis_max, pos - head_r, pos + head_r)
                }
                PanelType::Front | PanelType::Back => {
                    // Axis along Y, row across X
                    (pos - head_r, pos + head_r, axis_min, axis_max)
                }
                _ => (0.0, 0.0, 0.0, 0.0),
            };

            let b = BoundingBox::new(
                Point3::new(x0, y0, z_center - head_r),
                Point3::new(x1, y1, z_center + head_r),
            );

            parts.push(CratePart {
                id: format!("LAG-{}-{:02}", panel.name().to_uppercase(), i + 1),
                name: format!("Lag Screw ({}) {}", panel.name(), i + 1),
                category: PartCategory::Hardware,
                kind: CratePartKind::LagScrew {
                    panel,
                    index: i,
                    component: "screw".to_string(),
                },
                material: PartMaterial::Hardware { sku: sku.clone() },
                bounds: b,
                metadata: Some(format!("Axis {}, spacing target {:.2}\"", axis, spacing)),
            });
        }
    };

    add_for_panel(PanelType::Left);
    add_for_panel(PanelType::Right);
    add_for_panel(PanelType::Front);
    add_for_panel(PanelType::Back);

    parts
}

fn generate_decals(spec: &CrateSpec, geometry: &CrateGeometry) -> Vec<CratePart> {
    let mut parts: Vec<CratePart> = Vec::new();
    let t = 0.01_f32; // thin plate thickness (inches)
    let edge_offset = 2.0_f32;

    let overall_h = geometry.overall_height;

    // Sizes derived from the AutoCrate reference behavior (inches).
    let fragile = if overall_h <= 73.0 {
        (8.0_f32, 2.31_f32, "FRAGILE_STENCIL")
    } else {
        (12.0_f32, 3.50_f32, "FRAGILE_STENCIL")
    };
    let handling = if overall_h <= 37.0 {
        (3.0_f32, 8.25_f32, "HANDLING_SYMBOLS")
    } else {
        (4.0_f32, 11.0_f32, "HANDLING_SYMBOLS")
    };
    let autocrate_text = if overall_h <= 37.0 {
        (12.0_f32, 3.0_f32, "AUTOCRATE_TEXT")
    } else if overall_h <= 73.0 {
        (18.0_f32, 4.5_f32, "AUTOCRATE_TEXT")
    } else {
        (24.0_f32, 6.0_f32, "AUTOCRATE_TEXT")
    };

    // Helper to add a decal to an outward-facing panel surface.
    let mut add_on_panel =
        |panel: PanelType, kind: &str, dims: (f32, f32), placement: &str, text: Option<String>| {
            let bounds = match panel {
                PanelType::Front => &geometry.panels.front.bounds,
                PanelType::Back => &geometry.panels.back.bounds,
                PanelType::Left => &geometry.panels.left.bounds,
                PanelType::Right => &geometry.panels.right.bounds,
                PanelType::Top => &geometry.panels.top.bounds,
            };

            let (w, h) = dims;

            let (u_min, u_max, v_min, v_max, n, outward_positive) = match panel {
                PanelType::Front => (bounds.min.x, bounds.max.x, bounds.min.z, bounds.max.z, bounds.min.y, false),
                PanelType::Back => (bounds.min.x, bounds.max.x, bounds.min.z, bounds.max.z, bounds.max.y, true),
                PanelType::Left => (bounds.min.y, bounds.max.y, bounds.min.z, bounds.max.z, bounds.min.x, false),
                PanelType::Right => (bounds.min.y, bounds.max.y, bounds.min.z, bounds.max.z, bounds.max.x, true),
                PanelType::Top => (bounds.min.x, bounds.max.x, bounds.min.y, bounds.max.y, bounds.max.z, true),
            };

            let (u_center, v_center) = match placement {
                "center" => ((u_min + u_max) / 2.0, (v_min + v_max) / 2.0),
                "upper_right" => (
                    u_max - edge_offset - w / 2.0,
                    v_max - edge_offset - h / 2.0,
                ),
                "lower_left" => (
                    u_min + edge_offset + w / 2.0,
                    v_min + edge_offset + h / 2.0,
                ),
                _ => ((u_min + u_max) / 2.0, (v_min + v_max) / 2.0),
            };

            let (u0, u1) = (u_center - w / 2.0, u_center + w / 2.0);
            let (v0, v1) = (v_center - h / 2.0, v_center + h / 2.0);

            // Normal thickness goes outward from panel face.
            let (n0, n1) = if outward_positive { (n, n + t) } else { (n - t, n) };

            let (min, max) = match panel {
                PanelType::Front | PanelType::Back => (Point3::new(u0, n0, v0), Point3::new(u1, n1, v1)),
                PanelType::Left | PanelType::Right => (Point3::new(n0, u0, v0), Point3::new(n1, u1, v1)),
                PanelType::Top => (Point3::new(u0, v0, n0), Point3::new(u1, v1, n1)),
            };

            let idx = parts.len();
            parts.push(CratePart {
                id: format!("DECAL-{}-{}-{:02}", kind, panel.name().to_uppercase(), idx + 1),
                name: format!("{} Decal ({})", kind, panel.name()),
                category: PartCategory::Decal,
                kind: CratePartKind::Decal {
                    panel,
                    kind: kind.to_string(),
                    index: idx,
                },
                material: PartMaterial::Decal {
                    sku: kind.to_string(),
                    text,
                },
                bounds: BoundingBox::new(min, max),
                metadata: Some(format!("Placement: {}", placement)),
            });
        };

    // Fragile + handling + autocrate text: place on side and end panels (not top).
    for panel in [PanelType::Front, PanelType::Back, PanelType::Left, PanelType::Right] {
        if spec.markings.fragile_stencil {
            add_on_panel(panel, fragile.2, (fragile.0, fragile.1), "center", None);
        }
        if spec.markings.handling_symbols {
            add_on_panel(panel, handling.2, (handling.0, handling.1), "upper_right", None);
        }
        if spec.markings.autocrate_text {
            add_on_panel(panel, autocrate_text.2, (autocrate_text.0, autocrate_text.1), "center", None);
        }
    }

    // ISPM-15 marking (export compliance) — include if required and enabled.
    if spec.requirements.ispm15.required && spec.markings.ispm15_mark {
        let text = spec
            .requirements
            .ispm15
            .mark_text
            .clone()
            .or_else(|| match spec.requirements.ispm15.treatment {
                Some(super::types::Ismp15Treatment::HeatTreated) => Some("ISPM15 HT".to_string()),
                Some(super::types::Ismp15Treatment::MethylBromideFumigated) => Some("ISPM15 MB".to_string()),
                None => Some("ISPM15".to_string()),
            });

        // Place on two opposite panels for redundancy.
        add_on_panel(PanelType::Left, "ISPM15_MARK", (6.0, 4.0), "lower_left", text.clone());
        add_on_panel(PanelType::Right, "ISPM15_MARK", (6.0, 4.0), "lower_left", text);
    }

    parts
}


