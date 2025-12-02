//! STEP Part 21 file writer

use super::entities::*;
use super::primitives::*;
use std::io::{self, Write};

pub struct StepWriter {
    id_gen: EntityIdGenerator,
    entities: Vec<(EntityId, Box<dyn StepEntity>)>,
}

impl StepWriter {
    pub fn new() -> Self {
        Self {
            id_gen: EntityIdGenerator::new(),
            entities: Vec::new(),
        }
    }

    /// Add a cartesian point and return its ID
    pub fn add_point(&mut self, name: &str, x: f64, y: f64, z: f64) -> EntityId {
        let id = self.id_gen.next();
        let point = CartesianPoint {
            id,
            name: name.to_string(),
            coordinates: [x, y, z],
        };
        self.entities.push((id, Box::new(point)));
        id
    }

    /// Add a direction and return its ID
    pub fn add_direction(&mut self, name: &str, x: f64, y: f64, z: f64) -> EntityId {
        let id = self.id_gen.next();
        let dir = Direction {
            id,
            name: name.to_string(),
            ratios: [x, y, z],
        };
        self.entities.push((id, Box::new(dir)));
        id
    }

    /// Add a box (8 points for demonstration)
    pub fn add_box(&mut self, min: [f64; 3], max: [f64; 3]) {
        // 8 corner points
        self.add_point("", min[0], min[1], min[2]);
        self.add_point("", max[0], min[1], min[2]);
        self.add_point("", max[0], max[1], min[2]);
        self.add_point("", min[0], max[1], min[2]);
        self.add_point("", min[0], min[1], max[2]);
        self.add_point("", max[0], min[1], max[2]);
        self.add_point("", max[0], max[1], max[2]);
        self.add_point("", min[0], max[1], max[2]);
    }

    /// Write complete STEP file
    pub fn write_to<W: Write>(&self, mut writer: W) -> io::Result<()> {
        // Header
        writeln!(writer, "ISO-10303-21;")?;
        writeln!(writer, "HEADER;")?;
        writeln!(writer, "FILE_DESCRIPTION(('AutoCrate ASTM D6039 Crate'),'2;1');")?;
        writeln!(writer, "FILE_NAME('crate.step','2025-12-02T00:00:00',('AutoCrate'),('Antimony Labs'),'','','');")?;
        writeln!(writer, "FILE_SCHEMA(('AP242_MANAGED_MODEL_BASED_3D_ENGINEERING_MIM_LF'));")?;
        writeln!(writer, "ENDSEC;")?;

        // Data section
        writeln!(writer, "DATA;")?;

        for (id, entity) in &self.entities {
            entity.write_entity(*id, &mut writer)?;
        }

        writeln!(writer, "ENDSEC;")?;
        writeln!(writer, "END-ISO-10303-21;")?;

        Ok(())
    }

    /// Get STEP content as string
    pub fn to_string(&self) -> String {
        let mut buf = Vec::new();
        self.write_to(&mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }
}

impl Default for StepWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_writer_basic() {
        let mut writer = StepWriter::new();
        let p1 = writer.add_point("", 0.0, 0.0, 0.0);
        let p2 = writer.add_point("", 1.0, 1.0, 1.0);

        assert_eq!(p1, EntityId(1));
        assert_eq!(p2, EntityId(2));

        let output = writer.to_string();
        assert!(output.contains("ISO-10303-21"));
        assert!(output.contains("CARTESIAN_POINT"));
    }
}
