//! Color management and theme utilities
//!
//! Provides color primitives and conversion functions for:
//! - HSL/RGB/Hex color spaces
//! - Theme management
//! - Color interpolation for visualizations
//!

#![allow(clippy::excessive_precision)]
//! ## Traceability
//! - Used by: too.foo (boid rendering, fungal colors), helios (star colors), future CV projects
//! - Tests: test_hsl_rgb_conversion, test_color_interpolation, test_theme_palette

/// RGB color with 8-bit components
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Create from hex string (e.g., "#FF0000" or "FF0000")
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Self { r, g, b })
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Convert to CSS rgba string with alpha
    pub fn to_rgba_string(&self, alpha: f32) -> String {
        format!("rgba({}, {}, {}, {:.2})", self.r, self.g, self.b, alpha)
    }

    /// Convert to HSL
    pub fn to_hsl(&self) -> Hsl {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        let l = (max + min) / 2.0;

        if delta < 0.00001 {
            return Hsl::new(0.0, 0.0, l);
        }

        let s = if l < 0.5 {
            delta / (max + min)
        } else {
            delta / (2.0 - max - min)
        };

        let h = if (max - r).abs() < 0.00001 {
            ((g - b) / delta) % 6.0
        } else if (max - g).abs() < 0.00001 {
            (b - r) / delta + 2.0
        } else {
            (r - g) / delta + 4.0
        };

        let h = (h * 60.0 + 360.0) % 360.0;

        Hsl::new(h, s, l)
    }

    /// Linear interpolation between two colors
    pub fn lerp(&self, other: &Rgb, t: f32) -> Rgb {
        let t = t.clamp(0.0, 1.0);
        Rgb {
            r: (self.r as f32 + (other.r as f32 - self.r as f32) * t) as u8,
            g: (self.g as f32 + (other.g as f32 - self.g as f32) * t) as u8,
            b: (self.b as f32 + (other.b as f32 - self.b as f32) * t) as u8,
        }
    }

    /// Normalized RGB (0.0-1.0) for GPU/shader use
    pub fn to_normalized(&self) -> [f32; 3] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        ]
    }
}

impl Default for Rgb {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

/// HSL color (Hue, Saturation, Lightness)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hsl {
    /// Hue in degrees (0-360)
    pub h: f32,
    /// Saturation (0.0-1.0)
    pub s: f32,
    /// Lightness (0.0-1.0)
    pub l: f32,
}

impl Hsl {
    pub fn new(h: f32, s: f32, l: f32) -> Self {
        Self {
            h: h % 360.0,
            s: s.clamp(0.0, 1.0),
            l: l.clamp(0.0, 1.0),
        }
    }

    /// Create from integer values (h: 0-360, s: 0-100, l: 0-100)
    pub fn from_int(h: u16, s: u8, l: u8) -> Self {
        Self::new(h as f32, s as f32 / 100.0, l as f32 / 100.0)
    }

    /// Convert to RGB
    pub fn to_rgb(&self) -> Rgb {
        if self.s < 0.00001 {
            let v = (self.l * 255.0) as u8;
            return Rgb::new(v, v, v);
        }

        let q = if self.l < 0.5 {
            self.l * (1.0 + self.s)
        } else {
            self.l + self.s - self.l * self.s
        };
        let p = 2.0 * self.l - q;

        let h = self.h / 360.0;

        fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
            if t < 0.0 {
                t += 1.0;
            }
            if t > 1.0 {
                t -= 1.0;
            }
            if t < 1.0 / 6.0 {
                return p + (q - p) * 6.0 * t;
            }
            if t < 1.0 / 2.0 {
                return q;
            }
            if t < 2.0 / 3.0 {
                return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
            }
            p
        }

        let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
        let g = hue_to_rgb(p, q, h);
        let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

        Rgb::new(
            (r * 255.0).round() as u8,
            (g * 255.0).round() as u8,
            (b * 255.0).round() as u8,
        )
    }

    /// Convert to CSS hsl string
    pub fn to_hsl_string(&self) -> String {
        format!(
            "hsl({}, {}%, {}%)",
            self.h as u16,
            (self.s * 100.0) as u8,
            (self.l * 100.0) as u8
        )
    }

    /// Interpolate in HSL space (smoother for color transitions)
    pub fn lerp(&self, other: &Hsl, t: f32) -> Hsl {
        let t = t.clamp(0.0, 1.0);

        // Handle hue wrapping (take shortest path around color wheel)
        let mut h_diff = other.h - self.h;
        if h_diff > 180.0 {
            h_diff -= 360.0;
        } else if h_diff < -180.0 {
            h_diff += 360.0;
        }

        Hsl::new(
            (self.h + h_diff * t + 360.0) % 360.0,
            self.s + (other.s - self.s) * t,
            self.l + (other.l - self.l) * t,
        )
    }

    /// Adjust lightness (positive = lighter, negative = darker)
    pub fn adjust_lightness(&self, delta: f32) -> Hsl {
        Hsl::new(self.h, self.s, (self.l + delta).clamp(0.0, 1.0))
    }

    /// Adjust saturation
    pub fn adjust_saturation(&self, delta: f32) -> Hsl {
        Hsl::new(self.h, (self.s + delta).clamp(0.0, 1.0), self.l)
    }
}

impl Default for Hsl {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.5)
    }
}

/// Color gradient for visualization
pub struct Gradient {
    stops: Vec<(f32, Rgb)>,
}

impl Gradient {
    /// Create a gradient from color stops [(position, color), ...]
    /// Positions should be in range 0.0-1.0
    pub fn new(stops: Vec<(f32, Rgb)>) -> Self {
        let mut stops = stops;
        stops.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        Self { stops }
    }

    /// Sample the gradient at position t (0.0-1.0)
    pub fn sample(&self, t: f32) -> Rgb {
        let t = t.clamp(0.0, 1.0);

        if self.stops.is_empty() {
            return Rgb::default();
        }

        if t <= self.stops[0].0 {
            return self.stops[0].1;
        }

        if t >= self.stops[self.stops.len() - 1].0 {
            return self.stops[self.stops.len() - 1].1;
        }

        for i in 0..self.stops.len() - 1 {
            let (pos0, color0) = self.stops[i];
            let (pos1, color1) = self.stops[i + 1];

            if t >= pos0 && t <= pos1 {
                let local_t = (t - pos0) / (pos1 - pos0);
                return color0.lerp(&color1, local_t);
            }
        }

        self.stops[0].1
    }

    /// Create a heat map gradient (blue -> cyan -> green -> yellow -> red)
    pub fn heat_map() -> Self {
        Self::new(vec![
            (0.0, Rgb::new(0, 0, 255)),    // Blue
            (0.25, Rgb::new(0, 255, 255)), // Cyan
            (0.5, Rgb::new(0, 255, 0)),    // Green
            (0.75, Rgb::new(255, 255, 0)), // Yellow
            (1.0, Rgb::new(255, 0, 0)),    // Red
        ])
    }

    /// Create a viridis-like gradient (perceptually uniform)
    pub fn viridis() -> Self {
        Self::new(vec![
            (0.0, Rgb::new(68, 1, 84)),
            (0.25, Rgb::new(59, 82, 139)),
            (0.5, Rgb::new(33, 145, 140)),
            (0.75, Rgb::new(94, 201, 98)),
            (1.0, Rgb::new(253, 231, 37)),
        ])
    }
}

/// Pre-defined color themes
pub mod themes {
    use super::Rgb;

    /// Nature/ecosystem theme
    pub struct Nature;
    impl Nature {
        pub const HERBIVORE: Rgb = Rgb::new(0, 255, 180); // Cyan-green
        pub const CARNIVORE: Rgb = Rgb::new(255, 60, 60); // Red
        pub const SCAVENGER: Rgb = Rgb::new(200, 150, 50); // Gold
        pub const FOOD: Rgb = Rgb::new(100, 200, 50); // Green
        pub const DANGER: Rgb = Rgb::new(150, 0, 150); // Purple
        pub const BACKGROUND: Rgb = Rgb::new(10, 15, 25); // Dark blue
    }

    /// Space/astronomy theme
    pub struct Space;
    impl Space {
        pub const STAR_HOT: Rgb = Rgb::new(155, 176, 255); // Blue-white
        pub const STAR_WARM: Rgb = Rgb::new(255, 255, 220); // Yellow-white
        pub const STAR_COOL: Rgb = Rgb::new(255, 180, 100); // Orange
        pub const NEBULA: Rgb = Rgb::new(100, 50, 150); // Purple
        pub const VOID: Rgb = Rgb::new(5, 5, 15); // Near-black
    }

    /// Matrix/cyber theme
    pub struct Cyber;
    impl Cyber {
        pub const PRIMARY: Rgb = Rgb::new(0, 255, 136); // Neon green
        pub const SECONDARY: Rgb = Rgb::new(255, 0, 128); // Neon pink
        pub const ACCENT: Rgb = Rgb::new(0, 200, 255); // Cyan
        pub const GRID: Rgb = Rgb::new(30, 40, 50); // Dark gray
        pub const BACKGROUND: Rgb = Rgb::new(10, 12, 18); // Very dark
    }
}

/// Temperature to star color (for astronomy visualizations)
/// Uses simplified black-body approximation
pub fn temperature_to_color(kelvin: f32) -> Rgb {
    // Clamp to reasonable stellar range
    let temp = kelvin.clamp(1000.0, 40000.0);

    let (r, g, b);

    // Red channel
    if temp <= 6600.0 {
        r = 255.0;
    } else {
        r = 329.698727446 * ((temp / 100.0 - 60.0).powf(-0.1332047592));
    }

    // Green channel
    if temp <= 6600.0 {
        g = 99.4708025861 * (temp / 100.0).ln() - 161.1195681661;
    } else {
        g = 288.1221695283 * ((temp / 100.0 - 60.0).powf(-0.0755148492));
    }

    // Blue channel
    if temp >= 6600.0 {
        b = 255.0;
    } else if temp <= 1900.0 {
        b = 0.0;
    } else {
        b = 138.5177312231 * (temp / 100.0 - 10.0).ln() - 305.0447927307;
    }

    Rgb::new(
        r.clamp(0.0, 255.0) as u8,
        g.clamp(0.0, 255.0) as u8,
        b.clamp(0.0, 255.0) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_hex_roundtrip() {
        let color = Rgb::new(255, 128, 64);
        let hex = color.to_hex();
        let parsed = Rgb::from_hex(&hex).unwrap();
        assert_eq!(color, parsed);
    }

    #[test]
    fn test_hsl_rgb_conversion() {
        // Red
        let red_hsl = Hsl::new(0.0, 1.0, 0.5);
        let red_rgb = red_hsl.to_rgb();
        assert_eq!(red_rgb.r, 255);
        assert!(red_rgb.g < 5);
        assert!(red_rgb.b < 5);

        // Green
        let green_hsl = Hsl::new(120.0, 1.0, 0.5);
        let green_rgb = green_hsl.to_rgb();
        assert!(green_rgb.r < 5);
        assert_eq!(green_rgb.g, 255);
        assert!(green_rgb.b < 5);

        // Blue
        let blue_hsl = Hsl::new(240.0, 1.0, 0.5);
        let blue_rgb = blue_hsl.to_rgb();
        assert!(blue_rgb.r < 5);
        assert!(blue_rgb.g < 5);
        assert_eq!(blue_rgb.b, 255);
    }

    #[test]
    fn test_rgb_to_hsl_roundtrip() {
        let original = Rgb::new(128, 64, 200);
        let hsl = original.to_hsl();
        let back = hsl.to_rgb();

        // Allow small rounding errors
        assert!((original.r as i16 - back.r as i16).abs() <= 1);
        assert!((original.g as i16 - back.g as i16).abs() <= 1);
        assert!((original.b as i16 - back.b as i16).abs() <= 1);
    }

    #[test]
    fn test_color_interpolation() {
        let black = Rgb::new(0, 0, 0);
        let white = Rgb::new(255, 255, 255);

        let mid = black.lerp(&white, 0.5);
        assert!((mid.r as i16 - 127).abs() <= 1);
        assert!((mid.g as i16 - 127).abs() <= 1);
        assert!((mid.b as i16 - 127).abs() <= 1);
    }

    #[test]
    fn test_gradient_sampling() {
        let gradient = Gradient::heat_map();

        // Start should be blue
        let start = gradient.sample(0.0);
        assert_eq!(start.b, 255);

        // End should be red
        let end = gradient.sample(1.0);
        assert_eq!(end.r, 255);
    }

    #[test]
    fn test_temperature_to_color() {
        // Hot stars should be blue-ish
        let hot = temperature_to_color(30000.0);
        assert!(hot.b >= hot.r);

        // Cool stars should be red-ish
        let cool = temperature_to_color(3000.0);
        assert!(cool.r >= cool.b);

        // Sun-like should be yellow-white
        let sun = temperature_to_color(5778.0);
        assert!(sun.r > 200 && sun.g > 200);
    }

    #[test]
    fn test_hsl_lerp_wrapping() {
        // Test hue wrapping (red to blue via purple, not green)
        let red = Hsl::new(0.0, 1.0, 0.5);
        let blue = Hsl::new(240.0, 1.0, 0.5);

        let mid = red.lerp(&blue, 0.5);
        // Should go through purple (around 300) not green (120)
        assert!(mid.h > 100.0 && mid.h < 140.0 || mid.h > 280.0);
    }
}
