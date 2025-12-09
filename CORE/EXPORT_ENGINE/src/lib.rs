//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: lib.rs | CORE/EXPORT_ENGINE/src/lib.rs
//! PURPOSE: Export pipeline for various file formats
//! LAYER: CORE → EXPORT_ENGINE
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! EXPORT_ENGINE generates output files in various formats:
//! - Gerber X2 (PCB fabrication)
//! - PDF (documentation, schematics)
//! - STEP (3D CAD exchange) [TODO]
//! - G-code (CNC machining) [TODO]
//!
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │ ARCHITECTURE                                                                │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │                                                                             │
//! │   ExportEngine                                                              │
//! │       │                                                                     │
//! │       ├── GerberDocument        (DNA/export/gerber)                         │
//! │       ├── PdfDocument           (DNA/export/pdf)                            │
//! │       └── StepWriter            [TODO]                                      │
//! │                                                                             │
//! │   Export flow:                                                              │
//! │   1. Accept geometry/data from application                                  │
//! │   2. Transform to target coordinate system                                  │
//! │   3. Generate format-specific output                                        │
//! │   4. Write to file or return as bytes                                       │
//! │                                                                             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//!
//! DEPENDS ON:
//!   • DNA/export/gerber → Gerber X2 generation
//!   • DNA/export/pdf → PDF generation
//!
//! USED BY:
//!   • TOOLS/* → File export functionality
//!
//! ═══════════════════════════════════════════════════════════════════════════════

// ─────────────────────────────────────────────────────────────────────────────────
// CODE BELOW - Optimized for ML development
// ─────────────────────────────────────────────────────────────────────────────────

// Re-export Gerber types from DNA
pub use dna::export::gerber::{
    GerberDocument, GerberCommand, GerberUnit, ApertureDef, ApertureType,
};

// Re-export PDF types from DNA
pub use dna::export::pdf::{
    PdfDocument, PdfPage, TextAlign,
};

/// Export format enumeration
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportFormat {
    GerberX2,
    Pdf,
    Step,
    GCode,
}

/// Unit system for export
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportUnits {
    Millimeters,
    Inches,
}

/// Export configuration
#[derive(Clone, Debug)]
pub struct ExportConfig {
    pub format: ExportFormat,
    pub output_path: String,
    pub units: ExportUnits,
    pub scale: f64,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            format: ExportFormat::GerberX2,
            output_path: "output".to_string(),
            units: ExportUnits::Millimeters,
            scale: 1.0,
        }
    }
}

/// Check if a format is supported for export
pub fn is_format_supported(format: ExportFormat) -> bool {
    matches!(format, ExportFormat::GerberX2 | ExportFormat::Pdf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_support() {
        assert!(is_format_supported(ExportFormat::GerberX2));
        assert!(is_format_supported(ExportFormat::Pdf));
        assert!(!is_format_supported(ExportFormat::Step));
        assert!(!is_format_supported(ExportFormat::GCode));
    }

    #[test]
    fn test_default_config() {
        let config = ExportConfig::default();
        assert_eq!(config.format, ExportFormat::GerberX2);
        assert_eq!(config.scale, 1.0);
    }

    #[test]
    fn test_gerber_document_creation() {
        let doc = GerberDocument::new("Copper,L1,Top");
        // Verify document was created
        let _ = doc;
    }
}
