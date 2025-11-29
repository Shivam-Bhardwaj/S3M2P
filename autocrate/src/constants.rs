// AutoCrate constants - Rust port of crate-constants.ts
// All ASTM standard dimensions and structural requirements

use serde::{Deserialize, Serialize};

/// Lumber nominal sizes
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LumberSize {
    L1x4,
    L2x3,
    L2x4,
    L2x6,
    L2x8,
    L2x10,
    L2x12,
    L3x3,
    L3x4,
    L4x4,
    L4x6,
    L6x6,
    L8x8,
}

impl LumberSize {
    /// Get actual dimensions (height, width) in inches
    pub fn actual(&self) -> (f32, f32) {
        match self {
            LumberSize::L1x4 => (0.75, 3.5),
            LumberSize::L2x3 => (1.5, 2.5),
            LumberSize::L2x4 => (1.5, 3.5),
            LumberSize::L2x6 => (1.5, 5.5),
            LumberSize::L2x8 => (1.5, 7.25),
            LumberSize::L2x10 => (1.5, 9.25),
            LumberSize::L2x12 => (1.5, 11.25),
            LumberSize::L3x3 => (2.5, 2.5),
            LumberSize::L3x4 => (3.5, 2.5), // Oriented for forklift clearance
            LumberSize::L4x4 => (3.5, 3.5),
            LumberSize::L4x6 => (3.5, 5.5),
            LumberSize::L6x6 => (5.5, 5.5),
            LumberSize::L8x8 => (7.25, 7.25),
        }
    }

    /// Get nominal name
    pub fn name(&self) -> &'static str {
        match self {
            LumberSize::L1x4 => "1x4",
            LumberSize::L2x3 => "2x3",
            LumberSize::L2x4 => "2x4",
            LumberSize::L2x6 => "2x6",
            LumberSize::L2x8 => "2x8",
            LumberSize::L2x10 => "2x10",
            LumberSize::L2x12 => "2x12",
            LumberSize::L3x3 => "3x3",
            LumberSize::L3x4 => "3x4",
            LumberSize::L4x4 => "4x4",
            LumberSize::L4x6 => "4x6",
            LumberSize::L6x6 => "6x6",
            LumberSize::L8x8 => "8x8",
        }
    }
}

/// Plywood standards
pub mod plywood {
    pub const SHEET_WIDTH: f32 = 48.0;
    pub const SHEET_LENGTH: f32 = 96.0;
    pub const DEFAULT_THICKNESS: f32 = 0.25;
    pub const AVAILABLE_THICKNESSES: [f32; 5] = [0.25, 0.375, 0.5, 0.625, 0.75];
}

/// Skid standards
pub mod skid {
    pub const MIN_FORKLIFT_HEIGHT: f32 = 3.5;
    pub const LIGHTWEIGHT_WEIGHT_THRESHOLD: f32 = 4500.0;
    pub const TWO_SKIDS_MAX_WEIGHT: f32 = 3000.0;
    pub const THREE_SKIDS_MAX_WEIGHT: f32 = 10000.0;

    /// Get recommended skid count based on weight
    pub fn recommended_count(weight: f32) -> u8 {
        if weight <= TWO_SKIDS_MAX_WEIGHT { 2 }
        else if weight <= THREE_SKIDS_MAX_WEIGHT { 3 }
        else { 4 }
    }

    /// Check if weight qualifies for lightweight lumber
    pub fn is_lightweight(weight: f32) -> bool {
        weight < LIGHTWEIGHT_WEIGHT_THRESHOLD
    }
}

/// Cleat standards
pub mod cleat {
    pub const MAX_VERTICAL_SPACING: f32 = 24.0;
    pub const DEFAULT_WIDTH: f32 = 3.5;
    pub const DEFAULT_THICKNESS: f32 = 0.75;
}

/// Fastener standards
pub mod fastener {
    pub const KLIMP_MIN_SPACING: f32 = 16.0;
    pub const KLIMP_MAX_SPACING: f32 = 24.0;
    pub const LAG_DEFAULT_SPACING: f32 = 21.0;
    pub const LAG_MIN_SPACING: f32 = 18.0;
    pub const LAG_MAX_SPACING: f32 = 24.0;
}

/// Panel stop standards
pub mod panel_stop {
    pub const THICKNESS: f32 = 0.375;
    pub const WIDTH: f32 = 2.0;
    pub const EDGE_INSET: f32 = 0.0625; // 1/16"
}

/// Geometry standards
pub mod geometry {
    pub const SIDE_PANEL_GROUND_CLEARANCE: f32 = 0.25;
    pub const SIDE_PANEL_EDGE_CLEARANCE: f32 = 0.0625;
    pub const DEFAULT_PANEL_THICKNESS: f32 = 1.0;
    pub const STANDARD_TOLERANCE: f32 = 0.0625; // 1/16"
}

/// Validation rules
pub mod validation {
    pub const MIN_DIMENSION: f32 = 12.0;
    pub const MAX_DIMENSION: f32 = 130.0;
    pub const MIN_WEIGHT: f32 = 50.0;
    pub const MAX_WEIGHT: f32 = 60000.0;
    pub const MIN_CLEARANCE: f32 = 1.0;
    pub const MAX_SIDE_CLEARANCE: f32 = 12.0;
    pub const MAX_TOP_CLEARANCE: f32 = 24.0;
}

/// Convert decimal inches to fractional display
pub fn to_fractional_inches(inches: f32) -> String {
    const TOLERANCE: f32 = 0.0001;
    const FRACTIONS: [(f32, &str); 15] = [
        (1.0/16.0, "1/16"), (1.0/8.0, "1/8"), (3.0/16.0, "3/16"),
        (1.0/4.0, "1/4"), (5.0/16.0, "5/16"), (3.0/8.0, "3/8"),
        (7.0/16.0, "7/16"), (1.0/2.0, "1/2"), (9.0/16.0, "9/16"),
        (5.0/8.0, "5/8"), (11.0/16.0, "11/16"), (3.0/4.0, "3/4"),
        (13.0/16.0, "13/16"), (7.0/8.0, "7/8"), (15.0/16.0, "15/16"),
    ];

    // Whole number
    if (inches - inches.round()).abs() < TOLERANCE {
        return format!("{}", inches.round() as i32);
    }

    let whole = inches.floor() as i32;
    let frac = inches - whole as f32;

    for &(value, display) in &FRACTIONS {
        if (frac - value).abs() < TOLERANCE {
            return if whole > 0 {
                format!("{} {}", whole, display)
            } else {
                display.to_string()
            };
        }
    }

    // No clean fraction
    format!("{:.3}", inches)
}

/// Format dimension with suffix
pub fn format_dimension(inches: f32, suffix: &str) -> String {
    format!("{}{}", to_fractional_inches(inches), suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lumber_dimensions() {
        let (h, w) = LumberSize::L2x4.actual();
        assert!((h - 1.5).abs() < 0.001);
        assert!((w - 3.5).abs() < 0.001);
    }

    #[test]
    fn test_fractional_inches() {
        assert_eq!(to_fractional_inches(0.25), "1/4");
        assert_eq!(to_fractional_inches(1.5), "1 1/2");
        assert_eq!(to_fractional_inches(2.0), "2");
    }

    #[test]
    fn test_skid_count() {
        assert_eq!(skid::recommended_count(2000.0), 2);
        assert_eq!(skid::recommended_count(5000.0), 3);
        assert_eq!(skid::recommended_count(15000.0), 4);
    }
}
