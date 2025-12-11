//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: step.rs | DNA/src/export/step.rs
//! PURPOSE: Minimal STEP (ISO-10303-21) writer for NX-importable assemblies (inches)
//! MODIFIED: 2025-12-11
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════

use crate::autocrate::design::{CrateDesign, CratePart};
use crate::autocrate::geometry::BoundingBox;

/// STEP export options.
#[derive(Clone, Debug)]
pub struct StepExportOptions {
    pub product_name: String,
    /// If true, embeds basic PROPERTY_DEFINITION PMI for overall crate bounding box (inches).
    pub include_pmi: bool,
}

impl Default for StepExportOptions {
    fn default() -> Self {
        Self {
            product_name: "AUTOCRATE CRATE ASSEMBLY".to_string(),
            include_pmi: true,
        }
    }
}

/// Export a crate design to an AP242-like STEP Part-21 file.
///
/// Notes:
/// - Geometry is emitted as **B-Rep boxes** (MANIFOLD_SOLID_BREP) for each part.
/// - Assembly is built via SHAPE_REPRESENTATION relationships + transformations.
/// - Units: **inches** (conversion-based unit in STEP context).
pub fn export_step_ap242(design: &CrateDesign, options: &StepExportOptions) -> String {
    StepWriter::new(design, options.clone()).generate()
}

// ─────────────────────────────────────────────────────────────────────────────
// Internal writer (based on the proven approach in the reference AutoCrate TS repo)
// ─────────────────────────────────────────────────────────────────────────────

struct StepWriter<'a> {
    design: &'a CrateDesign,
    options: StepExportOptions,
    id: u32,
    data: Vec<String>,
}

impl<'a> StepWriter<'a> {
    fn new(design: &'a CrateDesign, options: StepExportOptions) -> Self {
        Self {
            design,
            options,
            id: 1,
            data: Vec::new(),
        }
    }

    fn next_id(&mut self) -> String {
        let id = self.id;
        self.id += 1;
        format!("#{}", id)
    }

    fn add(&mut self, entity: String) -> String {
        let id = self.next_id();
        self.data.push(format!("{id}={entity};"));
        id
    }

    fn escape(s: &str) -> String {
        s.replace('\'', "''")
    }

    fn header(&self) -> String {
        // Keep header deterministic (important for golden tests + reproducible manufacturing artifacts).
        let now = "1970-01-01T00:00:00Z".to_string();
        [
            "ISO-10303-21;".to_string(),
            "HEADER;".to_string(),
            "FILE_DESCRIPTION(('AutoCrate crate model'),'2;1');".to_string(),
            format!(
                "FILE_NAME('crate_model.step','{}',('AutoCrate'),('Antimony Labs'), 'S3M2P STEP Writer','S3M2P','');",
                now
            ),
            "FILE_SCHEMA(('AP242_MANAGED_MODEL_BASED_3D_ENGINEERING_MIM_LATEST'));".to_string(),
            "ENDSEC;".to_string(),
        ]
        .join("\n")
    }

    fn direction(&mut self, v: (i32, i32, i32)) -> String {
        self.add(format!("DIRECTION('',({},{},{}))", v.0, v.1, v.2))
    }

    fn cartesian_point(&mut self, p: (f64, f64, f64)) -> String {
        self.add(format!("CARTESIAN_POINT('',({:.6},{:.6},{:.6}))", p.0, p.1, p.2))
    }

    fn axis2_placement(&mut self, label: &str, origin: (f64, f64, f64)) -> String {
        let label = Self::escape(label);
        let p = self.cartesian_point(origin);
        let z = self.direction((0, 0, 1));
        let x = self.direction((1, 0, 0));
        self.add(format!("AXIS2_PLACEMENT_3D('{label}',{p},{z},{x})"))
    }

    fn create_contexts(&mut self, product_name: &str) -> StepContexts {
        let escaped = Self::escape(product_name);
        let app = self.add("APPLICATION_CONTEXT('mechanical design')".to_string());
        let _protocol = self.add(format!("APPLICATION_PROTOCOL_DEFINITION('international standard','ap242_managed_model_based_3d_engineering_mim_latest',2020,{app})"));
        let mech = self.add(format!("MECHANICAL_CONTEXT('',{app},'mechanical')"));
        let product_ctx = self.add(format!("PRODUCT_CONTEXT('{escaped}',{app},'design')"));
        let design_ctx = self.add(format!("DESIGN_CONTEXT('{escaped}',{app},'design')"));

        let plane_angle = self.add("(NAMED_UNIT(*)PLANE_ANGLE_UNIT()SI_UNIT($,.RADIAN.))".to_string());
        let solid_angle = self.add("(NAMED_UNIT(*)SI_UNIT($,.STERADIAN.)SOLID_ANGLE_UNIT())".to_string());
        // Inches via conversion-based unit (1 in = 25.4 mm).
        let base_mm = self.add("(LENGTH_UNIT()NAMED_UNIT(*)SI_UNIT(.MILLI.,.METRE.))".to_string());
        let inch_measure = self.add(format!(
            "LENGTH_MEASURE_WITH_UNIT(LENGTH_MEASURE(25.4),{base_mm})"
        ));
        let length_unit = self.add(format!(
            "(NAMED_UNIT(*)LENGTH_UNIT()CONVERSION_BASED_UNIT('INCH',{inch_measure}))"
        ));
        let uncertainty = self.add(format!(
            "UNCERTAINTY_MEASURE_WITH_UNIT(LENGTH_MEASURE(0.01),{length_unit},'distance accuracy','')"
        ));

        let geom_ctx = self.add(format!(
            "(GEOMETRIC_REPRESENTATION_CONTEXT(3)GLOBAL_UNIT_ASSIGNED_CONTEXT(({length_unit},{plane_angle},{solid_angle}))GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT(({uncertainty}))REPRESENTATION_CONTEXT('','3D'))"
        ));

        let product = self.add(format!("PRODUCT('{escaped}','{escaped}','',({mech}))"));
        let formation = self.add(format!("PRODUCT_DEFINITION_FORMATION('','',{product})"));
        let prod_def = self.add(format!("PRODUCT_DEFINITION('crate definition','',{formation},{design_ctx})"));
        let prod_def_shape = self.add(format!("PRODUCT_DEFINITION_SHAPE('','',{prod_def})"));

        let _ = app;
        let _ = product_ctx;
        StepContexts {
            mechanical_context: mech,
            design_context: design_ctx,
            length_unit,
            geom_context: geom_ctx,
            assembly_product_def: prod_def,
            assembly_product_def_shape: prod_def_shape,
        }
    }

    fn create_component_product(&mut self, name: &str, ctx: &StepContexts) -> ProductDefinition {
        let escaped = Self::escape(name);
        let product = self.add(format!(
            "PRODUCT('{escaped}','{escaped}','',({}))",
            ctx.mechanical_context
        ));
        let formation = self.add(format!("PRODUCT_DEFINITION_FORMATION('','',{product})"));
        let prod_def =
            self.add(format!("PRODUCT_DEFINITION('{escaped} definition','',{formation},{})", ctx.design_context));
        let prod_def_shape = self.add(format!("PRODUCT_DEFINITION_SHAPE('','',{prod_def})"));
        let _ = product;
        ProductDefinition {
            product_def: prod_def,
            product_def_shape: prod_def_shape,
        }
    }

    fn create_box_solid(&mut self, name: &str, bounds_in: &BoundingBox) -> Option<String> {
        // Convert bounds to a local box (0..dx, 0..dy, 0..dz) and return origin for placement.
        let s = bounds_in.size();
        let (w, l, h) = (s.x as f64, s.y as f64, s.z as f64);
        if w <= 1e-6 || l <= 1e-6 || h <= 1e-6 {
            return None;
        }

        // Vertices
        let vertices = [
            (0.0, 0.0, 0.0),
            (w, 0.0, 0.0),
            (w, l, 0.0),
            (0.0, l, 0.0),
            (0.0, 0.0, h),
            (w, 0.0, h),
            (w, l, h),
            (0.0, l, h),
        ];

        let v_ids: Vec<(String, String)> = vertices
            .iter()
            .map(|&(x, y, z)| {
                let p = self.add(format!("CARTESIAN_POINT('',({:.6},{:.6},{:.6}))", x, y, z));
                let v = self.add(format!("VERTEX_POINT('',{p})"));
                (p, v)
            })
            .collect();

        // Edge helper
        let x_pos = self.direction((1, 0, 0));
        let y_pos = self.direction((0, 1, 0));
        let z_pos = self.direction((0, 0, 1));
        let x_neg = self.direction((-1, 0, 0));
        let y_neg = self.direction((0, -1, 0));
        let z_neg = self.direction((0, 0, -1));

        let vector = |dir: &String, len: f64, this: &mut StepWriter| -> String {
            this.add(format!("VECTOR('',{dir},{:.6})", len))
        };

        let line = |p: &String, vec: &String, this: &mut StepWriter| -> String {
            this.add(format!("LINE('',{p},{vec})"))
        };

        let edge_curve = |vs: &String, ve: &String, ln: &String, this: &mut StepWriter| -> String {
            this.add(format!("EDGE_CURVE('',{vs},{ve},{ln},.T.)"))
        };

        let oriented = |edge: &String, ori: bool, this: &mut StepWriter| -> String {
            this.add(format!(
                "ORIENTED_EDGE('',*,*,{edge},{})",
                if ori { ".T." } else { ".F." }
            ))
        };

        // Edges: match the reference ordering.
        let e0 = edge_curve(&v_ids[0].1, &v_ids[1].1, &line(&v_ids[0].0, &vector(&x_pos, w, self), self), self);
        let e1 = edge_curve(&v_ids[1].1, &v_ids[2].1, &line(&v_ids[1].0, &vector(&y_pos, l, self), self), self);
        let e2 = edge_curve(&v_ids[3].1, &v_ids[2].1, &line(&v_ids[3].0, &vector(&x_pos, w, self), self), self);
        let e3 = edge_curve(&v_ids[0].1, &v_ids[3].1, &line(&v_ids[0].0, &vector(&y_pos, l, self), self), self);
        let e4 = edge_curve(&v_ids[4].1, &v_ids[5].1, &line(&v_ids[4].0, &vector(&x_pos, w, self), self), self);
        let e5 = edge_curve(&v_ids[5].1, &v_ids[6].1, &line(&v_ids[5].0, &vector(&y_pos, l, self), self), self);
        let e6 = edge_curve(&v_ids[7].1, &v_ids[6].1, &line(&v_ids[7].0, &vector(&x_pos, w, self), self), self);
        let e7 = edge_curve(&v_ids[4].1, &v_ids[7].1, &line(&v_ids[4].0, &vector(&y_pos, l, self), self), self);
        let e8 = edge_curve(&v_ids[0].1, &v_ids[4].1, &line(&v_ids[0].0, &vector(&z_pos, h, self), self), self);
        let e9 = edge_curve(&v_ids[1].1, &v_ids[5].1, &line(&v_ids[1].0, &vector(&z_pos, h, self), self), self);
        let e10 = edge_curve(&v_ids[2].1, &v_ids[6].1, &line(&v_ids[2].0, &vector(&z_pos, h, self), self), self);
        let e11 = edge_curve(&v_ids[3].1, &v_ids[7].1, &line(&v_ids[3].0, &vector(&z_pos, h, self), self), self);

        // Planes
        let point = |x: f64, y: f64, z: f64, this: &mut StepWriter| -> String {
            this.add(format!("CARTESIAN_POINT('',({:.6},{:.6},{:.6}))", x, y, z))
        };
        let axis2 = |p: &String, n: &String, r: &String, this: &mut StepWriter| -> String {
            this.add(format!("AXIS2_PLACEMENT_3D('',{p},{n},{r})"))
        };
        let plane = |p: &String, n: &String, r: &String, this: &mut StepWriter| -> String {
            let axis = axis2(p, n, r, this);
            this.add(format!("PLANE('',{axis})"))
        };
        let face = |edges: Vec<String>, pl: &String, this: &mut StepWriter| -> String {
            let loop_id = this.add(format!("EDGE_LOOP('',({}))", edges.join(",")));
            let bound = this.add(format!("FACE_OUTER_BOUND('',{loop_id},.T.)"));
            this.add(format!("ADVANCED_FACE('',({bound}),{pl},.T.)"))
        };

        let bottom = face(
            vec![
                oriented(&e0, true, self),
                oriented(&e1, true, self),
                oriented(&e2, false, self),
                oriented(&e3, false, self),
            ],
            &plane(&point(w / 2.0, l / 2.0, 0.0, self), &z_neg, &x_pos, self),
            self,
        );
        let top = face(
            vec![
                oriented(&e4, true, self),
                oriented(&e5, true, self),
                oriented(&e6, false, self),
                oriented(&e7, false, self),
            ],
            &plane(&point(w / 2.0, l / 2.0, h, self), &z_pos, &x_pos, self),
            self,
        );
        let front = face(
            vec![
                oriented(&e0, true, self),
                oriented(&e9, true, self),
                oriented(&e4, false, self),
                oriented(&e8, false, self),
            ],
            &plane(&point(w / 2.0, 0.0, h / 2.0, self), &y_neg, &x_pos, self),
            self,
        );
        let back = face(
            vec![
                oriented(&e11, true, self),
                oriented(&e6, true, self),
                oriented(&e10, false, self),
                oriented(&e2, false, self),
            ],
            &plane(&point(w / 2.0, l, h / 2.0, self), &y_pos, &x_pos, self),
            self,
        );
        let left = face(
            vec![
                oriented(&e3, true, self),
                oriented(&e11, true, self),
                oriented(&e7, false, self),
                oriented(&e8, false, self),
            ],
            &plane(&point(0.0, l / 2.0, h / 2.0, self), &x_neg, &y_pos, self),
            self,
        );
        let right = face(
            vec![
                oriented(&e9, true, self),
                oriented(&e5, true, self),
                oriented(&e10, false, self),
                oriented(&e1, false, self),
            ],
            &plane(&point(w, l / 2.0, h / 2.0, self), &x_pos, &y_pos, self),
            self,
        );

        let shell = self.add(format!("CLOSED_SHELL('',({bottom},{top},{front},{back},{left},{right}))"));
        let solid_name = Self::escape(name);
        let solid = self.add(format!("MANIFOLD_SOLID_BREP('{solid_name}',{shell})"));

        Some(solid)
    }

    fn compute_bbox(parts: &[CratePart]) -> Option<BoundingBox> {
        if parts.is_empty() {
            return None;
        }
        let mut min = parts[0].bounds.min;
        let mut max = parts[0].bounds.max;
        for p in parts.iter().skip(1) {
            min.x = min.x.min(p.bounds.min.x);
            min.y = min.y.min(p.bounds.min.y);
            min.z = min.z.min(p.bounds.min.z);
            max.x = max.x.max(p.bounds.max.x);
            max.y = max.y.max(p.bounds.max.y);
            max.z = max.z.max(p.bounds.max.z);
        }
        Some(BoundingBox::new(min, max))
    }

    fn add_bbox_pmi_in(&mut self, ctx: &StepContexts, bbox_in: &BoundingBox) {
        let size = bbox_in.size();
        let add_len = |label: &str, value: f64, this: &mut StepWriter| {
            let label = Self::escape(label);
            let measure = this.add(format!("LENGTH_MEASURE_WITH_UNIT(LENGTH_MEASURE({:.3}),{})", value, ctx.length_unit));
            let item = this.add(format!("MEASURE_REPRESENTATION_ITEM('{label}',{measure})"));
            let rep = this.add(format!("REPRESENTATION('{label}',({item}),{})", ctx.geom_context));
            let prop = this.add(format!("PROPERTY_DEFINITION('{label}','product characteristic',{})", ctx.assembly_product_def));
            this.add(format!("PROPERTY_DEFINITION_REPRESENTATION({prop},{rep})"));
        };
        add_len("overall_width_in", size.x as f64, self);
        add_len("overall_length_in", size.y as f64, self);
        add_len("overall_height_in", size.z as f64, self);
    }

    fn generate(mut self) -> String {
        let header = self.header();
        self.data.push("DATA;".to_string());

        let product_name = self.options.product_name.clone();
        let ctx = self.create_contexts(&product_name);

        // Build part products + shape representations.
        // For v1 we avoid aggressive grouping: each `CratePart` is its own component.
        let assembly_name = self.options.product_name.clone();
        let assembly_shape_name = Self::escape(&assembly_name);

        let mut child_placements: Vec<String> = Vec::new();
        let mut child_shape_reps: Vec<String> = Vec::new();
        let mut child_products: Vec<ProductDefinition> = Vec::new();
        let mut child_local_placements: Vec<String> = Vec::new();

        let mut parts: Vec<&CratePart> = self.design.parts.iter().collect();
        parts.sort_by(|a, b| a.id.cmp(&b.id));

        for (i, part) in parts.into_iter().enumerate() {
            // Coordinates are already in inches; STEP context defines INCH length units.
            let solid = match self.create_box_solid(&part.id, &part.bounds) {
                Some(s) => s,
                None => continue,
            };

            let product = self.create_component_product(&part.id, &ctx);
            let shape_rep = self.add(format!(
                "ADVANCED_BREP_SHAPE_REPRESENTATION('{}',({}),{})",
                Self::escape(&part.id),
                solid,
                ctx.geom_context
            ));
            self.add(format!(
                "SHAPE_DEFINITION_REPRESENTATION({}, {})",
                product.product_def_shape, shape_rep
            ));

            // Placement: origin at the min corner of the part (in inches)
            let origin = (
                part.bounds.min.x as f64,
                part.bounds.min.y as f64,
                part.bounds.min.z as f64,
            );
            let local = self.axis2_placement(&format!("{}_LOCAL", part.id), (0.0, 0.0, 0.0));
            let global = self.axis2_placement(&format!("{}_ASM_{}", part.id, i + 1), origin);

            child_products.push(product);
            child_shape_reps.push(shape_rep);
            child_local_placements.push(local);
            child_placements.push(global);
        }

        let items = if child_placements.is_empty() {
            "()".to_string()
        } else {
            format!("({})", child_placements.join(","))
        };

        let root_shape_rep = self.add(format!(
            "SHAPE_REPRESENTATION('{}',{},{})",
            assembly_shape_name, items, ctx.geom_context
        ));
        self.add(format!(
            "SHAPE_DEFINITION_REPRESENTATION({}, {})",
            ctx.assembly_product_def_shape, root_shape_rep
        ));

        // Wire each child into root via REPRESENTATION_RELATIONSHIP + transformation + NAUO.
        for idx in 0..child_products.len() {
            let prod = &child_products[idx];
            let child_shape = &child_shape_reps[idx];
            let child_local = &child_local_placements[idx];
            let child_global = &child_placements[idx];

            let transform = self.add(format!(
                "ITEM_DEFINED_TRANSFORMATION('{}_TRANSFORM_{}','',{},{})",
                prod.product_def, idx + 1, child_local, child_global
            ));
            let rel = self.add(format!(
                "( REPRESENTATION_RELATIONSHIP('{}','',{},{}) REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION({}) SHAPE_REPRESENTATION_RELATIONSHIP() )",
                Self::escape(&prod.product_def),
                root_shape_rep,
                child_shape,
                transform
            ));
            let occurrence_name = format!("NAUO_{}", idx + 1);
            let usage = self.add(format!(
                "NEXT_ASSEMBLY_USAGE_OCCURRENCE('{}','{}','',{}, {}, $)",
                occurrence_name,
                Self::escape(&prod.product_def),
                ctx.assembly_product_def,
                prod.product_def
            ));
            let usage_shape = self.add(format!("PRODUCT_DEFINITION_SHAPE('','',{usage})"));
            self.add(format!("CONTEXT_DEPENDENT_SHAPE_REPRESENTATION({rel},{usage_shape})"));
        }

        if self.options.include_pmi {
            if let Some(bbox) = Self::compute_bbox(&self.design.parts) {
                self.add_bbox_pmi_in(&ctx, &bbox);
            }
        }

        self.data.push("ENDSEC;".to_string());
        self.data.push("END-ISO-10303-21;".to_string());

        [header, self.data.join("\n")].join("\n")
    }
}

#[derive(Clone)]
struct StepContexts {
    mechanical_context: String,
    design_context: String,
    length_unit: String,
    geom_context: String,
    assembly_product_def: String,
    assembly_product_def_shape: String,
}

#[derive(Clone)]
struct ProductDefinition {
    product_def: String,
    product_def_shape: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::autocrate::{CrateDesign, CrateSpec};

    #[test]
    fn step_export_contains_header_and_some_entities() {
        let spec = CrateSpec::default();
        let design = CrateDesign::from_spec(&spec);
        let step = export_step_ap242(&design, &StepExportOptions::default());

        assert!(step.contains("ISO-10303-21;"));
        assert!(step.contains("FILE_SCHEMA"));
        assert!(step.contains("DATA;"));
        assert!(step.contains("END-ISO-10303-21;"));
        assert!(step.contains("MANIFOLD_SOLID_BREP"));
        assert!(step.contains("NEXT_ASSEMBLY_USAGE_OCCURRENCE"));
        assert!(step.contains("CONVERSION_BASED_UNIT('INCH'"));
    }

    #[test]
    fn step_export_is_deterministic_for_same_design() {
        let spec = CrateSpec::default();
        let design = CrateDesign::from_spec(&spec);
        let options = StepExportOptions::default();

        let a = export_step_ap242(&design, &options);
        let b = export_step_ap242(&design, &options);

        assert_eq!(a, b);
    }
}


