//! PDF 1.4 Generator - From Scratch
//!
//! Generates PDF files without external dependencies.
//! Supports text, lines, rectangles, and basic formatting.
//!
//! PDF Reference: ISO 32000-1:2008
//!
//! Structure:
//! ```text
//! %PDF-1.4
//! 1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj
//! 2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj
//! 3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [...] /Contents 4 0 R >> endobj
//! 4 0 obj << /Length ... >> stream ... endstream endobj
//! xref
//! trailer << /Size ... /Root 1 0 R >>
//! startxref
//! %%EOF
//! ```

use std::fmt::Write as FmtWrite;

/// PDF document builder
#[derive(Debug)]
pub struct PdfDocument {
    /// Page width in points (1 point = 1/72 inch)
    pub page_width: f64,
    /// Page height in points
    pub page_height: f64,
    /// Content streams for each page
    pages: Vec<PdfPage>,
}

/// A single PDF page
#[derive(Debug)]
pub struct PdfPage {
    /// Content stream commands
    content: String,
    /// Current font size
    font_size: f64,
    /// Current line width
    line_width: f64,
}

/// Text alignment options
#[derive(Clone, Copy, Debug)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

impl Default for PdfDocument {
    fn default() -> Self {
        // US Letter size: 8.5" x 11" = 612 x 792 points
        Self::new(612.0, 792.0)
    }
}

impl PdfDocument {
    /// Create a new PDF document with specified page size
    pub fn new(page_width: f64, page_height: f64) -> Self {
        Self {
            page_width,
            page_height,
            pages: vec![PdfPage::new()],
        }
    }

    /// Create an A4 document (210mm x 297mm)
    pub fn a4() -> Self {
        Self::new(595.28, 841.89)
    }

    /// Get the current page for drawing
    fn current_page(&mut self) -> &mut PdfPage {
        self.pages.last_mut().expect("No pages in document")
    }

    /// Add a new page
    pub fn add_page(&mut self) {
        self.pages.push(PdfPage::new());
    }

    /// Set font size for subsequent text
    pub fn set_font_size(&mut self, size: f64) {
        self.current_page().font_size = size;
    }

    /// Set line width for subsequent graphics
    pub fn set_line_width(&mut self, width: f64) {
        let page = self.current_page();
        page.line_width = width;
        writeln!(page.content, "{:.2} w", width).ok();
    }

    /// Set stroke color (RGB, 0-1 range)
    pub fn set_stroke_color(&mut self, r: f64, g: f64, b: f64) {
        let page = self.current_page();
        writeln!(page.content, "{:.3} {:.3} {:.3} RG", r, g, b).ok();
    }

    /// Set fill color (RGB, 0-1 range)
    pub fn set_fill_color(&mut self, r: f64, g: f64, b: f64) {
        let page = self.current_page();
        writeln!(page.content, "{:.3} {:.3} {:.3} rg", r, g, b).ok();
    }

    /// Draw text at position (x, y from bottom-left)
    pub fn draw_text(&mut self, x: f64, y: f64, text: &str) {
        let page = self.current_page();
        let font_size = page.font_size;
        let escaped = escape_pdf_string(text);

        writeln!(page.content, "BT").ok();
        writeln!(page.content, "/F1 {:.1} Tf", font_size).ok();
        writeln!(page.content, "{:.2} {:.2} Td", x, y).ok();
        writeln!(page.content, "({}) Tj", escaped).ok();
        writeln!(page.content, "ET").ok();
    }

    /// Draw text with alignment
    pub fn draw_text_aligned(&mut self, x: f64, y: f64, text: &str, align: TextAlign) {
        let page = self.current_page();
        let font_size = page.font_size;

        // Approximate text width (assuming ~0.5 width per character for Helvetica)
        let char_width = font_size * 0.5;
        let text_width = text.len() as f64 * char_width;

        let adjusted_x = match align {
            TextAlign::Left => x,
            TextAlign::Center => x - text_width / 2.0,
            TextAlign::Right => x - text_width,
        };

        self.draw_text(adjusted_x, y, text);
    }

    /// Draw a line from (x1, y1) to (x2, y2)
    pub fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64) {
        let page = self.current_page();
        writeln!(page.content, "{:.2} {:.2} m", x1, y1).ok();
        writeln!(page.content, "{:.2} {:.2} l", x2, y2).ok();
        writeln!(page.content, "S").ok();
    }

    /// Draw a rectangle (stroke only)
    pub fn draw_rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        let page = self.current_page();
        writeln!(page.content, "{:.2} {:.2} {:.2} {:.2} re S", x, y, width, height).ok();
    }

    /// Draw a filled rectangle
    pub fn fill_rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        let page = self.current_page();
        writeln!(page.content, "{:.2} {:.2} {:.2} {:.2} re f", x, y, width, height).ok();
    }

    /// Draw a filled and stroked rectangle
    pub fn fill_stroke_rect(&mut self, x: f64, y: f64, width: f64, height: f64) {
        let page = self.current_page();
        writeln!(page.content, "{:.2} {:.2} {:.2} {:.2} re B", x, y, width, height).ok();
    }

    /// Generate the PDF file content as bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut output = String::new();
        let mut offsets: Vec<usize> = Vec::new();

        // Header
        output.push_str("%PDF-1.4\n");
        // Binary marker - use valid UTF-8 that signals binary content
        output.push_str("%\u{00E2}\u{00E3}\u{00CF}\u{00D3}\n");

        // Object 1: Catalog
        offsets.push(output.len());
        output.push_str("1 0 obj\n");
        output.push_str("<< /Type /Catalog /Pages 2 0 R >>\n");
        output.push_str("endobj\n");

        // Object 2: Pages
        offsets.push(output.len());
        output.push_str("2 0 obj\n");
        output.push_str("<< /Type /Pages /Kids [");
        for i in 0..self.pages.len() {
            if i > 0 {
                output.push(' ');
            }
            write!(output, "{} 0 R", 3 + i * 2).ok();
        }
        write!(output, "] /Count {} >>\n", self.pages.len()).ok();
        output.push_str("endobj\n");

        // For each page: Page object + Content stream
        let mut next_obj = 3;
        for page in &self.pages {
            // Page object
            offsets.push(output.len());
            write!(output, "{} 0 obj\n", next_obj).ok();
            output.push_str("<< /Type /Page /Parent 2 0 R ");
            write!(
                output,
                "/MediaBox [0 0 {:.2} {:.2}] ",
                self.page_width, self.page_height
            )
            .ok();
            write!(output, "/Contents {} 0 R ", next_obj + 1).ok();
            output.push_str("/Resources << /Font << /F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> >> >> ");
            output.push_str(">>\n");
            output.push_str("endobj\n");
            next_obj += 1;

            // Content stream
            let content_bytes = page.content.as_bytes();
            offsets.push(output.len());
            write!(output, "{} 0 obj\n", next_obj).ok();
            write!(output, "<< /Length {} >>\n", content_bytes.len()).ok();
            output.push_str("stream\n");
            output.push_str(&page.content);
            output.push_str("endstream\n");
            output.push_str("endobj\n");
            next_obj += 1;
        }

        // Cross-reference table
        let xref_offset = output.len();
        output.push_str("xref\n");
        write!(output, "0 {}\n", next_obj).ok();
        output.push_str("0000000000 65535 f \n");
        for offset in &offsets {
            write!(output, "{:010} 00000 n \n", offset).ok();
        }

        // Trailer
        output.push_str("trailer\n");
        write!(output, "<< /Size {} /Root 1 0 R >>\n", next_obj).ok();
        output.push_str("startxref\n");
        write!(output, "{}\n", xref_offset).ok();
        output.push_str("%%EOF\n");

        output.into_bytes()
    }
}

impl PdfPage {
    fn new() -> Self {
        Self {
            content: String::new(),
            font_size: 12.0,
            line_width: 1.0,
        }
    }
}

/// Escape special characters in PDF strings
fn escape_pdf_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '(' => result.push_str("\\("),
            ')' => result.push_str("\\)"),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            _ => result.push(c),
        }
    }
    result
}

// ============================================================================
// PLL-Specific PDF Generation
// ============================================================================

use crate::pll::{PLLDesign, DividerConfig};

/// Generate a PDF report for a PLL design
pub fn generate_pll_report(design: &PLLDesign) -> Vec<u8> {
    let mut pdf = PdfDocument::default();

    // Title
    pdf.set_font_size(24.0);
    pdf.set_fill_color(0.0, 0.5, 0.4); // Teal
    pdf.draw_text_aligned(306.0, 750.0, "PLL Design Report", TextAlign::Center);

    // Subtitle
    pdf.set_font_size(12.0);
    pdf.set_fill_color(0.4, 0.4, 0.4);
    let arch_str = match design.divider_n {
        DividerConfig::IntegerN { .. } => "Integer-N",
        DividerConfig::FractionalN { .. } => "Fractional-N",
    };
    pdf.draw_text_aligned(306.0, 725.0, &format!("{} Phase-Locked Loop", arch_str), TextAlign::Center);

    // Horizontal rule
    pdf.set_stroke_color(0.0, 0.5, 0.4);
    pdf.set_line_width(2.0);
    pdf.draw_line(50.0, 710.0, 562.0, 710.0);

    // Section: Requirements
    let mut y = 680.0;
    pdf.set_font_size(14.0);
    pdf.set_fill_color(0.0, 0.0, 0.0);
    pdf.draw_text(50.0, y, "Design Requirements");
    y -= 5.0;
    pdf.set_line_width(0.5);
    pdf.draw_line(50.0, y, 200.0, y);

    pdf.set_font_size(10.0);
    y -= 20.0;
    pdf.draw_text(60.0, y, &format!("Reference Frequency: {:.2} MHz", design.requirements.ref_freq_hz / 1e6));
    y -= 15.0;
    pdf.draw_text(60.0, y, &format!("Output Frequency: {:.2} - {:.2} MHz",
        design.requirements.output_freq_min_hz / 1e6,
        design.requirements.output_freq_max_hz / 1e6));
    y -= 15.0;
    pdf.draw_text(60.0, y, &format!("Loop Bandwidth: {:.2} kHz", design.requirements.loop_bandwidth_hz / 1e3));
    y -= 15.0;
    pdf.draw_text(60.0, y, &format!("Target Phase Margin: {:.1} deg", design.requirements.phase_margin_deg));

    // Section: Divider Configuration
    y -= 30.0;
    pdf.set_font_size(14.0);
    pdf.draw_text(50.0, y, "Divider Configuration");
    y -= 5.0;
    pdf.draw_line(50.0, y, 200.0, y);

    pdf.set_font_size(10.0);
    y -= 20.0;
    pdf.draw_text(60.0, y, &format!("Reference Divider (R): {}", design.divider_r));
    y -= 15.0;

    match &design.divider_n {
        DividerConfig::IntegerN { n, prescaler } => {
            pdf.draw_text(60.0, y, &format!("Feedback Divider (N): {}", n));
            if let Some(p) = prescaler {
                y -= 15.0;
                pdf.draw_text(60.0, y, &format!("Prescaler: {}", p));
            }
        }
        DividerConfig::FractionalN { n_int, n_frac, modulus, modulator_order } => {
            pdf.draw_text(60.0, y, &format!("Integer Part (N_INT): {}", n_int));
            y -= 15.0;
            pdf.draw_text(60.0, y, &format!("Fractional Part: {}/{}", n_frac, modulus));
            y -= 15.0;
            pdf.draw_text(60.0, y, &format!("Sigma-Delta Order: {}", modulator_order));
        }
    }
    y -= 15.0;
    pdf.draw_text(60.0, y, &format!("PFD Frequency: {:.2} MHz", design.pfd_freq_hz / 1e6));

    // Section: Loop Filter
    y -= 30.0;
    pdf.set_font_size(14.0);
    pdf.draw_text(50.0, y, "Loop Filter (2nd Order Passive)");
    y -= 5.0;
    pdf.draw_line(50.0, y, 250.0, y);

    pdf.set_font_size(10.0);
    y -= 20.0;
    pdf.draw_text(60.0, y, &format!("C1: {:.2} pF", design.loop_filter.c1_pf));
    y -= 15.0;
    pdf.draw_text(60.0, y, &format!("R1: {:.2} ohms", design.loop_filter.r1_ohms));
    y -= 15.0;
    pdf.draw_text(60.0, y, &format!("C2: {:.2} pF", design.loop_filter.c2_pf));

    // Section: Performance
    y -= 30.0;
    pdf.set_font_size(14.0);
    pdf.draw_text(50.0, y, "Stability Analysis");
    y -= 5.0;
    pdf.draw_line(50.0, y, 200.0, y);

    pdf.set_font_size(10.0);
    y -= 20.0;
    pdf.draw_text(60.0, y, &format!("Phase Margin: {:.1} deg", design.performance.phase_margin_deg));
    y -= 15.0;
    pdf.draw_text(60.0, y, &format!("Gain Margin: {:.1} dB", design.performance.gain_margin_db));
    y -= 15.0;
    pdf.draw_text(60.0, y, &format!("Crossover Frequency: {:.2} kHz", design.performance.crossover_freq_hz / 1e3));
    y -= 15.0;
    pdf.draw_text(60.0, y, &format!("Estimated Lock Time: {:.1} us", design.performance.lock_time_us));

    // Section: System Parameters
    y -= 30.0;
    pdf.set_font_size(14.0);
    pdf.draw_text(50.0, y, "System Parameters");
    y -= 5.0;
    pdf.draw_line(50.0, y, 200.0, y);

    pdf.set_font_size(10.0);
    y -= 20.0;
    pdf.draw_text(60.0, y, &format!("Charge Pump Current: {:.2} uA", design.charge_pump_current_ua));
    y -= 15.0;
    pdf.draw_text(60.0, y, &format!("VCO Gain (Kvco): {:.2} MHz/V", design.vco_gain_mhz_per_v));

    // Footer
    pdf.set_font_size(8.0);
    pdf.set_fill_color(0.5, 0.5, 0.5);
    pdf.draw_text_aligned(306.0, 30.0, "Generated by PLL Designer - too.foo", TextAlign::Center);

    pdf.to_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_creation() {
        let mut pdf = PdfDocument::default();

        pdf.set_font_size(24.0);
        pdf.draw_text(100.0, 700.0, "Hello, PDF!");

        pdf.set_font_size(12.0);
        pdf.draw_text(100.0, 680.0, "This is a test document.");

        pdf.set_stroke_color(1.0, 0.0, 0.0);
        pdf.set_line_width(2.0);
        pdf.draw_line(100.0, 670.0, 500.0, 670.0);

        pdf.set_fill_color(0.0, 0.0, 1.0);
        pdf.fill_rect(100.0, 600.0, 100.0, 50.0);

        let bytes = pdf.to_bytes();
        assert!(bytes.len() > 0);

        // Check PDF header
        assert!(bytes.starts_with(b"%PDF-1.4"));

        // Check PDF footer
        let footer = String::from_utf8_lossy(&bytes[bytes.len()-10..]);
        assert!(footer.contains("%%EOF"));
    }

    #[test]
    fn test_escape_pdf_string() {
        assert_eq!(escape_pdf_string("Hello"), "Hello");
        assert_eq!(escape_pdf_string("(test)"), "\\(test\\)");
        assert_eq!(escape_pdf_string("back\\slash"), "back\\\\slash");
    }

    #[test]
    fn test_pll_report_generation() {
        use crate::pll::{PLLRequirements, PLLArchitecture, design_pll};

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
        let pdf_bytes = generate_pll_report(&design);

        assert!(pdf_bytes.len() > 1000); // Should be a reasonable size
        assert!(pdf_bytes.starts_with(b"%PDF-1.4"));
    }
}
