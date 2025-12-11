//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: reports.rs | DNA/src/autocrate/reports.rs
//! PURPOSE: BOM + Cut List generation (CSV) from canonical `CrateDesign`
//! MODIFIED: 2025-12-11
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════

use super::design::{CrateDesign, CratePartKind, PartCategory, PartMaterial};

#[derive(Clone, Debug, PartialEq)]
pub struct BomRow {
    pub item: String,
    pub size: Option<String>,
    pub quantity: u32,
    pub material: String,
    pub note: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CutListRow {
    pub item: String,
    pub material: String,
    pub nominal: Option<String>,
    pub length_in: Option<f32>,
    pub width_in: Option<f32>,
    pub height_in: Option<f32>,
    pub thickness_in: Option<f32>,
    pub quantity: u32,
    pub note: Option<String>,
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn round2(v: f32) -> f32 {
    (v * 100.0).round() / 100.0
}

fn bbox_dims(desc: &super::geometry::BoundingBox) -> (f32, f32, f32) {
    let s = desc.size();
    (s.x.abs(), s.y.abs(), s.z.abs())
}

fn face_dims_for_sheet_part(desc: &super::geometry::BoundingBox) -> (f32, f32) {
    // Treat the two largest dimensions as the "cut face" dimensions.
    let (a, b, c) = bbox_dims(desc);
    let mut dims = [a, b, c];
    dims.sort_by(|x, y| y.partial_cmp(x).unwrap_or(std::cmp::Ordering::Equal));
    (dims[0], dims[1])
}

/// Generate an aggregated BOM from the canonical `CrateDesign`.
pub fn generate_bom(design: &CrateDesign) -> Vec<BomRow> {
    use std::collections::BTreeMap;

    let mut rows: Vec<BomRow> = Vec::new();

    // Plywood sheets (rough estimate via area, no nesting in v1)
    let mut plywood_area = 0.0_f32;
    let mut plywood_thicknesses: Vec<f32> = Vec::new();
    for part in &design.parts {
        if part.category != PartCategory::Plywood {
            continue;
        }
        let (w, h) = face_dims_for_sheet_part(&part.bounds);
        plywood_area += w * h;
        if let PartMaterial::Plywood { thickness_in } = part.material {
            plywood_thicknesses.push(thickness_in);
        }
    }
    if plywood_area > 0.0 {
        let sheet_area = 48.0_f32 * 96.0_f32;
        let sheets = (plywood_area / sheet_area).ceil().max(1.0) as u32;
        plywood_thicknesses.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        plywood_thicknesses.dedup_by(|a, b| (*a - *b).abs() < 1e-3);

        rows.push(BomRow {
            item: "Plywood Sheet (48x96)".to_string(),
            size: Some("48\" x 96\"".to_string()),
            quantity: sheets,
            material: "Plywood".to_string(),
            note: Some(format!(
                "Estimated from total cut area {:.0} in^2. Thicknesses present: {:?}",
                plywood_area, plywood_thicknesses
            )),
        });
    }

    // Lumber + hardware + decals: aggregate by a stable key.
    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct Key {
        bucket: String,
        item: String,
        size: String,
        material: String,
    }

    let mut counts: BTreeMap<Key, (u32, Option<String>)> = BTreeMap::new();

    for part in &design.parts {
        match (&part.category, &part.material, &part.kind) {
            (PartCategory::Lumber, PartMaterial::Lumber { nominal }, kind) => {
                let (dx, dy, dz) = bbox_dims(&part.bounds);
                let length = round2(dx.max(dy).max(dz));
                let item = match kind {
                    CratePartKind::Skid { .. } => "Skid",
                    CratePartKind::Floorboard { .. } => "Floorboard",
                    CratePartKind::Cleat { .. } => "Cleat",
                    _ => "Lumber",
                }
                .to_string();

                let size = format!("{} x {:.2}\"", nominal.name(), length);
                let key = Key {
                    bucket: "lumber".to_string(),
                    item,
                    size: size.clone(),
                    material: "Lumber".to_string(),
                };
                let entry = counts.entry(key).or_insert((0, None));
                entry.0 += 1;
            }

            (PartCategory::Hardware, PartMaterial::Hardware { sku }, _kind) => {
                let key = Key {
                    bucket: "hardware".to_string(),
                    item: sku.clone(),
                    size: "".to_string(),
                    material: "Hardware".to_string(),
                };
                let entry = counts.entry(key).or_insert((0, None));
                entry.0 += 1;
            }

            (PartCategory::Decal, PartMaterial::Decal { sku, .. }, _kind) => {
                let key = Key {
                    bucket: "decals".to_string(),
                    item: sku.clone(),
                    size: "".to_string(),
                    material: "Decal".to_string(),
                };
                let entry = counts.entry(key).or_insert((0, None));
                entry.0 += 1;
            }

            _ => {}
        }
    }

    for (key, (qty, note)) in counts {
        rows.push(BomRow {
            item: key.item,
            size: if key.size.is_empty() { None } else { Some(key.size) },
            quantity: qty,
            material: key.material,
            note,
        });
    }

    rows
}

/// Generate an aggregated cut list (lumber + plywood cut parts).
pub fn generate_cut_list(design: &CrateDesign) -> Vec<CutListRow> {
    use std::collections::BTreeMap;

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct Key {
        category: String,
        item: String,
        nominal: String,
        a: u32,
        b: u32,
        c: u32,
        t: u32,
    }

    let mut map: BTreeMap<Key, u32> = BTreeMap::new();
    let mut notes: BTreeMap<Key, String> = BTreeMap::new();

    for part in &design.parts {
        match (&part.category, &part.material, &part.kind) {
            (PartCategory::Lumber, PartMaterial::Lumber { nominal }, kind) => {
                let (dx, dy, dz) = bbox_dims(&part.bounds);
                let length = round2(dx.max(dy).max(dz));
                let item = match kind {
                    CratePartKind::Skid { .. } => "Skid",
                    CratePartKind::Floorboard { .. } => "Floorboard",
                    CratePartKind::Cleat { .. } => "Cleat",
                    _ => "Lumber",
                }
                .to_string();

                let key = Key {
                    category: "lumber".to_string(),
                    item: item.clone(),
                    nominal: nominal.name().to_string(),
                    a: (length * 100.0) as u32,
                    b: 0,
                    c: 0,
                    t: 0,
                };
                *map.entry(key.clone()).or_insert(0) += 1;
                notes.entry(key).or_insert_with(|| item);
            }

            (PartCategory::Plywood, PartMaterial::Plywood { thickness_in }, kind) => {
                let (w, h) = face_dims_for_sheet_part(&part.bounds);
                let (w, h, t) = (round2(w), round2(h), round2(*thickness_in));
                let item = match kind {
                    CratePartKind::Panel { panel } => format!("Panel {}", panel.name()),
                    CratePartKind::PanelStop { .. } => "Panel Stop".to_string(),
                    _ => "Plywood".to_string(),
                };

                let key = Key {
                    category: "plywood".to_string(),
                    item: item.clone(),
                    nominal: "".to_string(),
                    a: (w * 100.0) as u32,
                    b: (h * 100.0) as u32,
                    c: 0,
                    t: (t * 100.0) as u32,
                };
                *map.entry(key.clone()).or_insert(0) += 1;
                notes.entry(key).or_insert_with(|| item);
            }

            _ => {}
        }
    }

    let mut out: Vec<CutListRow> = Vec::new();

    for (key, qty) in map {
        if key.category == "lumber" {
            out.push(CutListRow {
                item: key.item.clone(),
                material: "Lumber".to_string(),
                nominal: Some(key.nominal.clone()),
                length_in: Some((key.a as f32) / 100.0),
                width_in: None,
                height_in: None,
                thickness_in: None,
                quantity: qty,
                note: notes.get(&key).cloned(),
            });
        } else if key.category == "plywood" {
            out.push(CutListRow {
                item: key.item.clone(),
                material: "Plywood".to_string(),
                nominal: None,
                length_in: None,
                width_in: Some((key.a as f32) / 100.0),
                height_in: Some((key.b as f32) / 100.0),
                thickness_in: Some((key.t as f32) / 100.0),
                quantity: qty,
                note: notes.get(&key).cloned(),
            });
        }
    }

    out
}

pub fn bom_to_csv(rows: &[BomRow]) -> String {
    let mut out = String::new();
    out.push_str("item,size,quantity,material,note\n");
    for r in rows {
        let item = csv_escape(&r.item);
        let size = csv_escape(r.size.as_deref().unwrap_or(""));
        let qty = r.quantity.to_string();
        let material = csv_escape(&r.material);
        let note = csv_escape(r.note.as_deref().unwrap_or(""));
        out.push_str(&format!("{},{},{},{},{}\n", item, size, qty, material, note));
    }
    out
}

pub fn cut_list_to_csv(rows: &[CutListRow]) -> String {
    let mut out = String::new();
    out.push_str("item,material,nominal,length_in,width_in,height_in,thickness_in,quantity,note\n");
    for r in rows {
        let item = csv_escape(&r.item);
        let material = csv_escape(&r.material);
        let nominal = csv_escape(r.nominal.as_deref().unwrap_or(""));
        let length = r
            .length_in
            .map(|v| format!("{:.2}", v))
            .unwrap_or_default();
        let width = r
            .width_in
            .map(|v| format!("{:.2}", v))
            .unwrap_or_default();
        let height = r
            .height_in
            .map(|v| format!("{:.2}", v))
            .unwrap_or_default();
        let thickness = r
            .thickness_in
            .map(|v| format!("{:.3}", v))
            .unwrap_or_default();
        let qty = r.quantity.to_string();
        let note = csv_escape(r.note.as_deref().unwrap_or(""));
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            item, material, nominal, length, width, height, thickness, qty, note
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autocrate::CrateDesign;
    use crate::autocrate::CrateSpec;

    #[test]
    fn bom_and_cut_list_are_non_empty_for_default_design() {
        let spec = CrateSpec::default();
        let design = CrateDesign::from_spec(&spec);

        let bom = generate_bom(&design);
        let cut = generate_cut_list(&design);

        assert!(!bom.is_empty());
        assert!(!cut.is_empty());

        // Should include at least some hardware/decals by default.
        let bom_csv = bom_to_csv(&bom);
        assert!(bom_csv.contains("KLIMP_FASTENER"));
        assert!(bom_csv.contains("LAG_SCREW_0.38x3.00"));
        assert!(bom_csv.contains("FRAGILE_STENCIL"));
        assert!(bom_csv.contains("HANDLING_SYMBOLS"));
    }
}


