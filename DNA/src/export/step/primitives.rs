//! STEP geometric primitives

use super::entities::*;
use std::io::{self, Write};

/// CARTESIAN_POINT - 3D point
pub struct CartesianPoint {
    pub id: EntityId,
    pub name: String,
    pub coordinates: [f64; 3],
}

impl StepEntity for CartesianPoint {
    fn entity_name(&self) -> &'static str {
        "CARTESIAN_POINT"
    }

    fn write_attributes(&self, w: &mut dyn Write) -> io::Result<()> {
        write_step_string(&self.name, w)?;
        write!(w, ",({:.6},{:.6},{:.6})",
            self.coordinates[0],
            self.coordinates[1],
            self.coordinates[2]
        )
    }
}

/// DIRECTION - normalized direction vector
pub struct Direction {
    pub id: EntityId,
    pub name: String,
    pub ratios: [f64; 3],
}

impl StepEntity for Direction {
    fn entity_name(&self) -> &'static str {
        "DIRECTION"
    }

    fn write_attributes(&self, w: &mut dyn Write) -> io::Result<()> {
        write_step_string(&self.name, w)?;
        write!(w, ",({:.6},{:.6},{:.6})",
            self.ratios[0],
            self.ratios[1],
            self.ratios[2]
        )
    }
}

/// AXIS2_PLACEMENT_3D - local coordinate system
pub struct Axis2Placement3D {
    pub id: EntityId,
    pub name: String,
    pub location: EntityId,      // CARTESIAN_POINT
    pub axis: Option<EntityId>,  // DIRECTION (Z axis)
    pub ref_direction: Option<EntityId>, // DIRECTION (X axis)
}

impl StepEntity for Axis2Placement3D {
    fn entity_name(&self) -> &'static str {
        "AXIS2_PLACEMENT_3D"
    }

    fn write_attributes(&self, w: &mut dyn Write) -> io::Result<()> {
        write_step_string(&self.name, w)?;
        write!(w, ",{},", self.location)?;
        write_step_optional(self.axis, w)?;
        write!(w, ",")?;
        write_step_optional(self.ref_direction, w)
    }

    fn references(&self) -> Vec<EntityId> {
        let mut refs = vec![self.location];
        if let Some(axis) = self.axis {
            refs.push(axis);
        }
        if let Some(ref_dir) = self.ref_direction {
            refs.push(ref_dir);
        }
        refs
    }
}
