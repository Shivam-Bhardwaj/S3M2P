//! Gerber X2 Generator - From Scratch
//!
//! Generates Gerber RS-274X / X2 files without external dependencies.
//! Used for PCB fabrication of loop filter circuits.
//!
//! Gerber X2 Reference: UCAMCO specification rev. 2023.08
//!
//! Format overview:
//! ```text
//! %TF.FileFunction,Copper,L1,Top*%
//! %FSLAX36Y36*%
//! %MOIN*%
//! D10*
//! X0Y0D03*
//! X1000000Y0D01*
//! M02*
//! ```

/// Gerber document builder
#[derive(Debug)]
pub struct GerberDocument {
    /// File function (e.g., "Copper,L1,Top")
    file_function: String,
    /// Commands in the document
    commands: Vec<GerberCommand>,
    /// Aperture definitions
    apertures: Vec<ApertureDef>,
    /// Current aperture number
    current_aperture: u32,
    /// Unit (inches or mm)
    unit: GerberUnit,
    /// Format: integer digits, decimal digits
    format: (u8, u8),
}

/// Gerber command types
#[derive(Debug, Clone)]
pub enum GerberCommand {
    /// Move to position (D02)
    Move { x: i64, y: i64 },
    /// Draw line to position (D01)
    Line { x: i64, y: i64 },
    /// Flash aperture at position (D03)
    Flash { x: i64, y: i64 },
    /// Select aperture
    SelectAperture(u32),
    /// Region start
    RegionStart,
    /// Region end
    RegionEnd,
}

/// Aperture definition
#[derive(Debug, Clone)]
pub struct ApertureDef {
    /// Aperture number (D10+)
    number: u32,
    /// Aperture type
    aperture_type: ApertureType,
}

/// Aperture types
#[derive(Debug, Clone)]
pub enum ApertureType {
    /// Circle with diameter
    Circle { diameter: f64 },
    /// Rectangle with width and height
    Rectangle { width: f64, height: f64 },
    /// Obround (pill shape)
    Obround { width: f64, height: f64 },
}

/// Unit system
#[derive(Debug, Clone, Copy)]
pub enum GerberUnit {
    Inches,
    Millimeters,
}

impl Default for GerberDocument {
    fn default() -> Self {
        Self::new("Copper,L1,Top")
    }
}

impl GerberDocument {
    /// Create a new Gerber document
    pub fn new(file_function: &str) -> Self {
        Self {
            file_function: file_function.to_string(),
            commands: Vec::new(),
            apertures: Vec::new(),
            current_aperture: 10,
            unit: GerberUnit::Millimeters,
            format: (3, 6), // 3 integer, 6 decimal digits
        }
    }

    /// Set unit to millimeters
    pub fn set_unit_mm(&mut self) {
        self.unit = GerberUnit::Millimeters;
    }

    /// Set unit to inches
    pub fn set_unit_inches(&mut self) {
        self.unit = GerberUnit::Inches;
    }

    /// Add a circular aperture
    pub fn add_circle_aperture(&mut self, diameter: f64) -> u32 {
        let number = self.current_aperture;
        self.apertures.push(ApertureDef {
            number,
            aperture_type: ApertureType::Circle { diameter },
        });
        self.current_aperture += 1;
        number
    }

    /// Add a rectangular aperture
    pub fn add_rect_aperture(&mut self, width: f64, height: f64) -> u32 {
        let number = self.current_aperture;
        self.apertures.push(ApertureDef {
            number,
            aperture_type: ApertureType::Rectangle { width, height },
        });
        self.current_aperture += 1;
        number
    }

    /// Select an aperture
    pub fn select_aperture(&mut self, aperture: u32) {
        self.commands.push(GerberCommand::SelectAperture(aperture));
    }

    /// Convert coordinate to Gerber integer format
    fn coord_to_gerber(&self, coord: f64) -> i64 {
        // Format is X.XXXXXX (6 decimal places)
        (coord * 1_000_000.0).round() as i64
    }

    /// Move to position (without drawing)
    pub fn move_to(&mut self, x: f64, y: f64) {
        self.commands.push(GerberCommand::Move {
            x: self.coord_to_gerber(x),
            y: self.coord_to_gerber(y),
        });
    }

    /// Draw line to position
    pub fn line_to(&mut self, x: f64, y: f64) {
        self.commands.push(GerberCommand::Line {
            x: self.coord_to_gerber(x),
            y: self.coord_to_gerber(y),
        });
    }

    /// Flash aperture at position
    pub fn flash(&mut self, x: f64, y: f64) {
        self.commands.push(GerberCommand::Flash {
            x: self.coord_to_gerber(x),
            y: self.coord_to_gerber(y),
        });
    }

    /// Start a region (filled polygon)
    pub fn region_start(&mut self) {
        self.commands.push(GerberCommand::RegionStart);
    }

    /// End a region
    pub fn region_end(&mut self) {
        self.commands.push(GerberCommand::RegionEnd);
    }

    /// Draw a rectangle outline
    pub fn draw_rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.move_to(x, y);
        self.line_to(x + width, y);
        self.line_to(x + width, y + height);
        self.line_to(x, y + height);
        self.line_to(x, y);
    }

    /// Draw a filled rectangle
    pub fn fill_rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.region_start();
        self.move_to(x, y);
        self.line_to(x + width, y);
        self.line_to(x + width, y + height);
        self.line_to(x, y + height);
        self.line_to(x, y);
        self.region_end();
    }
}

impl std::fmt::Display for GerberDocument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // X2 attributes
        writeln!(f, "%TF.GenerationSoftware,too.foo,PLL Designer,1.0*%")?;
        writeln!(f, "%TF.FileFunction,{}*%", self.file_function)?;

        // Format specification
        let (int_digits, dec_digits) = self.format;
        writeln!(
            f,
            "%FSLAX{}{}Y{}{}*%",
            int_digits, dec_digits, int_digits, dec_digits
        )?;

        // Units
        match self.unit {
            GerberUnit::Inches => writeln!(f, "%MOIN*%")?,
            GerberUnit::Millimeters => writeln!(f, "%MOMM*%")?,
        };

        // Aperture definitions
        for aperture in &self.apertures {
            let def = match &aperture.aperture_type {
                ApertureType::Circle { diameter } => format!("C,{:.6}", diameter),
                ApertureType::Rectangle { width, height } => {
                    format!("R,{:.6}X{:.6}", width, height)
                }
                ApertureType::Obround { width, height } => format!("O,{:.6}X{:.6}", width, height),
            };
            writeln!(f, "%ADD{:02}{}*%", aperture.number, def)?;
        }

        // Set linear interpolation mode
        writeln!(f, "G01*")?;

        // Commands
        for cmd in &self.commands {
            match cmd {
                GerberCommand::SelectAperture(n) => {
                    writeln!(f, "D{:02}*", n)?;
                }
                GerberCommand::Move { x, y } => {
                    writeln!(f, "X{}Y{}D02*", x, y)?;
                }
                GerberCommand::Line { x, y } => {
                    writeln!(f, "X{}Y{}D01*", x, y)?;
                }
                GerberCommand::Flash { x, y } => {
                    writeln!(f, "X{}Y{}D03*", x, y)?;
                }
                GerberCommand::RegionStart => {
                    writeln!(f, "G36*")?;
                }
                GerberCommand::RegionEnd => {
                    writeln!(f, "G37*")?;
                }
            }
        }

        // End of file
        writeln!(f, "M02*")?;

        Ok(())
    }
}

impl GerberDocument {
    /// Generate the Gerber file content as bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

// ============================================================================
// PLL Loop Filter Gerber Generation
// ============================================================================

use crate::pll::PLLDesign;

/// Generate a Gerber file for a PLL loop filter footprint
///
/// Creates a simple 2-layer PCB footprint with:
/// - SMD pads for C1, R1, C2 (0805 footprint)
/// - Traces connecting them
/// - Via to ground plane
pub fn generate_loop_filter_gerber(design: &PLLDesign) -> GerberDocument {
    let mut gerber = GerberDocument::new("Copper,L1,Top");
    gerber.set_unit_mm();

    // Define apertures
    let trace_ap = gerber.add_circle_aperture(0.3); // 0.3mm trace
    let pad_ap = gerber.add_rect_aperture(1.2, 1.4); // 0805 pad
    let via_ap = gerber.add_circle_aperture(0.6); // Via pad

    // Component positions (mm)
    // Layout: [CP_OUT]--[C1]--+--[R1]--[C2]--[VCO_IN]
    //                        |
    //                       GND

    let y_center = 5.0;
    let cp_out_x = 2.0;
    let c1_x = 5.0;
    let r1_x = 10.0;
    let c2_x = 15.0;
    let vco_in_x = 18.0;
    let gnd_y = 2.0;

    // Draw C1 pads
    gerber.select_aperture(pad_ap);
    gerber.flash(c1_x - 0.9, y_center); // Left pad
    gerber.flash(c1_x + 0.9, y_center); // Right pad

    // Draw R1 pads
    gerber.flash(r1_x - 0.9, y_center);
    gerber.flash(r1_x + 0.9, y_center);

    // Draw C2 pads
    gerber.flash(c2_x - 0.9, y_center);
    gerber.flash(c2_x + 0.9, y_center);

    // Draw traces
    gerber.select_aperture(trace_ap);

    // CP_OUT to C1 left pad
    gerber.move_to(cp_out_x, y_center);
    gerber.line_to(c1_x - 0.9, y_center);

    // C1 right pad to junction point
    let junction_x = c1_x + 0.9 + 1.0;
    gerber.move_to(c1_x + 0.9, y_center);
    gerber.line_to(junction_x, y_center);

    // Junction to R1 left pad
    gerber.line_to(r1_x - 0.9, y_center);

    // R1 right pad to C2 left pad
    gerber.move_to(r1_x + 0.9, y_center);
    gerber.line_to(c2_x - 0.9, y_center);

    // C2 right pad to VCO_IN
    gerber.move_to(c2_x + 0.9, y_center);
    gerber.line_to(vco_in_x, y_center);

    // Junction to ground via
    gerber.move_to(junction_x, y_center);
    gerber.line_to(junction_x, gnd_y);

    // Ground via
    gerber.select_aperture(via_ap);
    gerber.flash(junction_x, gnd_y);

    // Add component designators as comment attributes
    // (In real Gerber X2, we'd add %TO.C attributes)

    // Add component values as X2 attributes would go here
    let _c1_val = design.loop_filter.c1_pf;
    let _r1_val = design.loop_filter.r1_ohms;
    let _c2_val = design.loop_filter.c2_pf;

    gerber
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gerber_creation() {
        let mut gerber = GerberDocument::new("Copper,L1,Top");
        gerber.set_unit_mm();

        let circle = gerber.add_circle_aperture(0.5);
        let rect = gerber.add_rect_aperture(1.0, 2.0);

        gerber.select_aperture(circle);
        gerber.flash(10.0, 20.0);

        gerber.select_aperture(rect);
        gerber.move_to(0.0, 0.0);
        gerber.line_to(100.0, 0.0);

        let output = gerber.to_string();
        assert!(output.contains("%TF.FileFunction,Copper,L1,Top*%"));
        assert!(output.contains("%MOMM*%"));
        assert!(output.contains("%ADD10C,0.500000*%"));
        assert!(output.contains("M02*"));
    }

    #[test]
    fn test_loop_filter_gerber() {
        use crate::pll::{design_pll, PLLArchitecture, PLLRequirements};

        let requirements = PLLRequirements {
            ref_freq_hz: 10e6,
            output_freq_min_hz: 2.4e9,
            output_freq_max_hz: 2.5e9,
            loop_bandwidth_hz: 100e3,
            phase_margin_deg: 45.0,
            architecture: PLLArchitecture::IntegerN,
            supply_voltage: 3.3,
        };

        let design = design_pll(&requirements).expect("Design should succeed");
        let gerber = generate_loop_filter_gerber(&design);
        let output = gerber.to_string();

        // Check basic structure
        assert!(output.contains("%TF.GenerationSoftware,too.foo,PLL Designer,1.0*%"));
        assert!(output.contains("D10*")); // Aperture selection
        assert!(output.contains("D03*")); // Flash commands
        assert!(output.contains("D01*")); // Line commands
        assert!(output.contains("M02*")); // End of file
    }

    #[test]
    fn test_coordinate_conversion() {
        let gerber = GerberDocument::default();
        // 10.5 mm should become 10500000
        assert_eq!(gerber.coord_to_gerber(10.5), 10500000);
    }
}
