//! STEP (ISO 10303-21) file format writer
//!
//! Provides export to STEP AP242 format with B-rep geometry and PMI/GD&T annotations.
//!
//! # Architecture
//!
//! - `entities` - Entity ID management and StepEntity trait
//! - `writer` - Part 21 file serialization (header + data sections)
//! - `primitives` - Geometric primitives (points, directions, axes)
//! - `topology` - B-rep topology (vertices, edges, faces, solids)
//! - `brep` - High-level B-rep construction helpers
//! - `product` - Product structure (context, definitions, representations)
//! - `pmi` - PMI entities (dimensions, annotations, material specs)
//! - `gdt` - GD&T entities (geometric tolerances, datums, FCFs)
//!
//! # Example
//!
//! ```rust,ignore
//! use dna::export::step::StepWriter;
//!
//! let mut writer = StepWriter::new();
//! writer.add_box(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 1.0, 1.0));
//! writer.write_to_file("output.step")?;
//! ```

pub mod entities;
pub mod writer;
pub mod primitives;
pub mod topology;
pub mod brep;
pub mod product;
pub mod pmi;
pub mod gdt;

pub use entities::{EntityId, EntityIdGenerator, StepEntity};
pub use writer::StepWriter;
