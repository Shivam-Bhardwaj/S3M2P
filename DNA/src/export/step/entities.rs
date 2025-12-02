//! STEP entity ID management and core trait

use std::fmt::{self, Display};
use std::io::{self, Write};

/// Entity ID - corresponds to `#N` in STEP files
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntityId(pub u64);

impl Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}", self.0)
    }
}

impl EntityId {
    pub const INVALID: Self = EntityId(0);

    #[inline]
    pub fn is_valid(&self) -> bool {
        self.0 > 0
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

/// Generator for monotonically increasing entity IDs
pub struct EntityIdGenerator {
    next_id: u64,
}

impl EntityIdGenerator {
    pub fn new() -> Self {
        Self { next_id: 1 } // STEP IDs start at 1
    }

    pub fn next(&mut self) -> EntityId {
        let id = EntityId(self.next_id);
        self.next_id += 1;
        id
    }

    pub fn current(&self) -> EntityId {
        EntityId(self.next_id - 1)
    }
}

impl Default for EntityIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Core trait for all STEP entities
pub trait StepEntity {
    /// Entity type name in UPPERCASE (e.g., "CARTESIAN_POINT")
    fn entity_name(&self) -> &'static str;

    /// Serialize entity attributes to STEP format
    /// Format: (attr1, attr2, attr3, ...)
    fn write_attributes(&self, w: &mut dyn Write) -> io::Result<()>;

    /// Get all referenced entity IDs for topological sorting
    fn references(&self) -> Vec<EntityId> {
        Vec::new()
    }

    /// Write complete entity line: #ID = ENTITY_NAME(attrs);
    fn write_entity(&self, id: EntityId, w: &mut dyn Write) -> io::Result<()> {
        write!(w, "{} = {}(", id, self.entity_name())?;
        self.write_attributes(w)?;
        writeln!(w, ");")
    }
}

/// Helper for writing STEP string values (single-quoted, escaped)
pub fn write_step_string(s: &str, w: &mut dyn Write) -> io::Result<()> {
    write!(w, "'")?;
    for ch in s.chars() {
        match ch {
            '\'' => write!(w, "''")?, // Escape single quotes
            '\\' => write!(w, "\\\\")?,
            _ => write!(w, "{}", ch)?,
        }
    }
    write!(w, "'")
}

/// Write STEP real number (f32/f64 with proper formatting)
pub fn write_step_real(val: f32, w: &mut dyn Write) -> io::Result<()> {
    // STEP uses scientific notation with capital E
    write!(w, "{:.6E}", val)
}

/// Write STEP optional value (use $ for undefined/null)
pub fn write_step_optional<T: Display>(opt: Option<T>, w: &mut dyn Write) -> io::Result<()> {
    match opt {
        Some(ref val) => write!(w, "{}", val),
        None => write!(w, "$"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_id_display() {
        let id = EntityId(42);
        assert_eq!(format!("{}", id), "#42");
    }

    #[test]
    fn test_entity_id_generator() {
        let mut gen = EntityIdGenerator::new();
        assert_eq!(gen.next(), EntityId(1));
        assert_eq!(gen.next(), EntityId(2));
        assert_eq!(gen.current(), EntityId(2));
    }

    #[test]
    fn test_write_step_string() {
        let mut buf = Vec::new();
        write_step_string("Hello 'World'", &mut buf).unwrap();
        assert_eq!(String::from_utf8(buf).unwrap(), "'Hello ''World'''");
    }

    #[test]
    fn test_write_step_real() {
        let mut buf = Vec::new();
        write_step_real(3.14159, &mut buf).unwrap();
        let result = String::from_utf8(buf).unwrap();
        assert!(result.contains("E"));
        assert!(result.starts_with("3.14"));
    }
}
