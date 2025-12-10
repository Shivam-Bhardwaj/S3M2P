//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: mod.rs | DNA/src/world/stars/mod.rs
//! PURPOSE: Star database with 3D positions for celestial visualization
//! CREATED: 2025-12-09
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! # Star Database
//!
//! Provides 3D positions for stars using Hipparcos catalog data with parallax.
//! Positions are in Heliocentric Inertial (HCI) frame in AU.
//!
//! ## Coordinate Conversion
//!
//! Stars are cataloged with:
//! - RA (Right Ascension) in degrees (0-360)
//! - Dec (Declination) in degrees (-90 to +90)
//! - Parallax in milliarcseconds (mas)
//!
//! We convert to Cartesian XYZ in AU:
//! ```text
//! distance_pc = 1000 / parallax_mas
//! distance_au = distance_pc * 206264.806 (parsecs to AU)
//!
//! x = distance * cos(dec) * cos(ra)
//! y = distance * cos(dec) * sin(ra)
//! z = distance * sin(dec)
//! ```
//!
//! ## References
//!
//! - Hipparcos Catalogue (ESA SP-1200)
//! - IAU SOFA Library for coordinate transforms
//!
//! ═══════════════════════════════════════════════════════════════════════════════

use glam::DVec3;
use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────────
// CONSTANTS
// ─────────────────────────────────────────────────────────────────────────────────

/// Parsecs to AU conversion factor
pub const PC_TO_AU: f64 = 206264.806;

/// Light-years to AU
pub const LY_TO_AU: f64 = 63241.077;

/// Parsecs to light-years
pub const PC_TO_LY: f64 = 3.26156;

/// Maximum stars in fixed-size database (avoid heap allocation in hot paths)
pub const MAX_STARS: usize = 10000;

// ─────────────────────────────────────────────────────────────────────────────────
// STAR STRUCT
// ─────────────────────────────────────────────────────────────────────────────────

/// A single star with 3D position and visual properties
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Star {
    /// Hipparcos catalog ID (unique identifier)
    pub hip_id: u32,

    /// Common name (e.g., "Sirius", "Betelgeuse")
    /// Empty string if no common name
    pub name: String,

    /// Position in HCI frame (AU from Sun)
    /// Derived from RA/Dec/parallax
    pub position: DVec3,

    /// Distance from Sun in parsecs
    pub distance_pc: f64,

    /// Apparent magnitude (brightness as seen from Earth)
    /// Lower = brighter. Sirius = -1.46, faintest naked eye ~6.0
    pub magnitude: f64,

    /// B-V color index (spectral color)
    /// -0.3 = blue (hot), 0 = white, 0.6 = yellow, 1.4 = orange, 2.0 = red
    pub color_bv: f64,

    /// Constellation abbreviation (e.g., "Ori" for Orion)
    pub constellation: String,
}

impl Star {
    /// Create a new star from catalog data
    ///
    /// # Arguments
    /// * `hip_id` - Hipparcos catalog ID
    /// * `name` - Common name (can be empty)
    /// * `ra_deg` - Right Ascension in degrees (0-360)
    /// * `dec_deg` - Declination in degrees (-90 to +90)
    /// * `parallax_mas` - Parallax in milliarcseconds
    /// * `magnitude` - Apparent magnitude
    /// * `color_bv` - B-V color index
    /// * `constellation` - Constellation abbreviation
    #[allow(clippy::too_many_arguments)]
    pub fn from_catalog(
        hip_id: u32,
        name: &str,
        ra_deg: f64,
        dec_deg: f64,
        parallax_mas: f64,
        magnitude: f64,
        color_bv: f64,
        constellation: &str,
    ) -> Self {
        // Calculate distance
        let distance_pc = if parallax_mas > 0.0 {
            1000.0 / parallax_mas
        } else {
            10000.0 // Default to 10 kpc for invalid parallax
        };

        // Convert to Cartesian position in AU
        let position = ra_dec_distance_to_cartesian(ra_deg, dec_deg, distance_pc);

        Self {
            hip_id,
            name: name.to_string(),
            position,
            distance_pc,
            magnitude,
            color_bv,
            constellation: constellation.to_string(),
        }
    }

    /// Get distance in light-years
    #[inline]
    pub fn distance_ly(&self) -> f64 {
        self.distance_pc * PC_TO_LY
    }

    /// Get distance in AU
    #[inline]
    pub fn distance_au(&self) -> f64 {
        self.distance_pc * PC_TO_AU
    }

    /// Get color as RGB (0-255) from B-V index
    ///
    /// Uses a simplified temperature-to-color mapping
    pub fn color_rgb(&self) -> (u8, u8, u8) {
        bv_to_rgb(self.color_bv)
    }

    /// Get apparent size for rendering based on magnitude
    ///
    /// Returns a size multiplier (brighter = larger)
    pub fn apparent_size(&self) -> f64 {
        // Magnitude scale: each 5 magnitudes = 100x brightness
        // mag -1.46 (Sirius) -> size ~4
        // mag 0 -> size ~2.5
        // mag 6 -> size ~0.5
        let base_size = 2.512_f64.powf(-self.magnitude / 2.5);
        base_size.clamp(0.3, 6.0)
    }
}

// ─────────────────────────────────────────────────────────────────────────────────
// STAR DATABASE
// ─────────────────────────────────────────────────────────────────────────────────

/// Catalog raw data format (for JSON deserialization)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub hip: u32,
    #[serde(default)]
    pub name: String,
    pub ra: f64,
    pub dec: f64,
    pub parallax: f64,
    pub mag: f64,
    #[serde(default)]
    pub bv: f64,
    #[serde(default)]
    pub con: String,
}

/// Database of stars for visualization
#[derive(Clone, Debug)]
pub struct StarDatabase {
    /// Fixed-size array of stars (SoA-style for cache efficiency)
    stars: Vec<Star>,

    /// Index by HIP ID for fast lookup
    hip_index: std::collections::HashMap<u32, usize>,
}

impl Default for StarDatabase {
    fn default() -> Self {
        Self::new()
    }
}

impl StarDatabase {
    /// Create empty database
    pub fn new() -> Self {
        Self {
            stars: Vec::with_capacity(MAX_STARS),
            hip_index: std::collections::HashMap::with_capacity(MAX_STARS),
        }
    }

    /// Load stars from JSON catalog data
    pub fn load_from_json(json_data: &str) -> Result<Self, String> {
        let entries: Vec<CatalogEntry> =
            serde_json::from_str(json_data).map_err(|e| format!("JSON parse error: {}", e))?;

        let mut db = Self::new();
        for entry in entries {
            if db.stars.len() >= MAX_STARS {
                break;
            }
            let star = Star::from_catalog(
                entry.hip,
                &entry.name,
                entry.ra,
                entry.dec,
                entry.parallax,
                entry.mag,
                entry.bv,
                &entry.con,
            );
            db.hip_index.insert(entry.hip, db.stars.len());
            db.stars.push(star);
        }

        Ok(db)
    }

    /// Get number of stars in database
    #[inline]
    pub fn len(&self) -> usize {
        self.stars.len()
    }

    /// Check if database is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.stars.is_empty()
    }

    /// Get star by index
    #[inline]
    pub fn get(&self, index: usize) -> Option<&Star> {
        self.stars.get(index)
    }

    /// Get star by Hipparcos ID
    pub fn get_by_hip(&self, hip_id: u32) -> Option<&Star> {
        self.hip_index
            .get(&hip_id)
            .and_then(|&idx| self.stars.get(idx))
    }

    /// Iterate over all stars
    pub fn iter(&self) -> impl Iterator<Item = &Star> {
        self.stars.iter()
    }

    /// Get stars brighter than given magnitude
    pub fn brighter_than(&self, mag_limit: f64) -> impl Iterator<Item = &Star> {
        self.stars.iter().filter(move |s| s.magnitude < mag_limit)
    }

    /// Get stars in a constellation
    pub fn in_constellation(&self, constellation: &str) -> impl Iterator<Item = &Star> {
        let con = constellation.to_string();
        self.stars.iter().filter(move |s| s.constellation == con)
    }

    /// Find stars within angular distance from a direction
    ///
    /// # Arguments
    /// * `center` - Direction vector (will be normalized)
    /// * `radius_deg` - Angular radius in degrees
    pub fn within_angle(&self, center: DVec3, radius_deg: f64) -> Vec<&Star> {
        let center_norm = center.normalize();
        let cos_radius = (radius_deg * std::f64::consts::PI / 180.0).cos();

        self.stars
            .iter()
            .filter(|s| {
                let star_dir = s.position.normalize();
                star_dir.dot(center_norm) >= cos_radius
            })
            .collect()
    }
}

// ─────────────────────────────────────────────────────────────────────────────────
// COORDINATE CONVERSION
// ─────────────────────────────────────────────────────────────────────────────────

/// Convert RA/Dec/distance to Cartesian XYZ in AU
///
/// # Arguments
/// * `ra_deg` - Right Ascension in degrees (0-360)
/// * `dec_deg` - Declination in degrees (-90 to +90)
/// * `distance_pc` - Distance in parsecs
///
/// # Returns
/// Position in HCI frame (X toward vernal equinox, Z toward north celestial pole)
pub fn ra_dec_distance_to_cartesian(ra_deg: f64, dec_deg: f64, distance_pc: f64) -> DVec3 {
    let ra_rad = ra_deg * std::f64::consts::PI / 180.0;
    let dec_rad = dec_deg * std::f64::consts::PI / 180.0;
    let distance_au = distance_pc * PC_TO_AU;

    let cos_dec = dec_rad.cos();
    DVec3::new(
        distance_au * cos_dec * ra_rad.cos(),
        distance_au * cos_dec * ra_rad.sin(),
        distance_au * dec_rad.sin(),
    )
}

/// Convert B-V color index to RGB values
///
/// Based on Ballesteros formula (2012) for blackbody approximation
pub fn bv_to_rgb(bv: f64) -> (u8, u8, u8) {
    // Clamp B-V to valid range
    let bv = bv.clamp(-0.4, 2.0);

    // Temperature approximation from B-V
    let temp = 4600.0 * (1.0 / (0.92 * bv + 1.7) + 1.0 / (0.92 * bv + 0.62));

    // Convert temperature to RGB (simplified Planck curve approximation)
    let (r, g, b) = temp_to_rgb(temp);

    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

/// Convert blackbody temperature to RGB (0-1 range)
fn temp_to_rgb(temp: f64) -> (f64, f64, f64) {
    let temp = temp.clamp(1000.0, 40000.0);

    let (r, g, b) = if temp < 6600.0 {
        let r = 1.0;
        let g = (0.39 * (temp / 100.0 - 10.0).ln() - 0.634).clamp(0.0, 1.0);
        let b = if temp <= 1900.0 {
            0.0
        } else {
            (0.543 * (temp / 100.0 - 10.0).ln() - 1.681).clamp(0.0, 1.0)
        };
        (r, g, b)
    } else {
        let r = (1.269 * (temp / 100.0 - 60.0).powf(-0.1332)).clamp(0.0, 1.0);
        let g = (1.144 * (temp / 100.0 - 60.0).powf(-0.0755)).clamp(0.0, 1.0);
        let b = 1.0;
        (r, g, b)
    };

    (r, g, b)
}

// ─────────────────────────────────────────────────────────────────────────────────
// BUILT-IN BRIGHT STARS (fallback when no catalog loaded)
// ─────────────────────────────────────────────────────────────────────────────────

/// Create a database with the ~50 brightest stars (visible to naked eye)
/// Used as fallback when full Hipparcos catalog is not available
pub fn create_bright_stars() -> StarDatabase {
    let mut db = StarDatabase::new();

    // Top 50 brightest stars (approximate data)
    // Format: HIP, name, RA, Dec, parallax_mas, magnitude, B-V, constellation
    let bright_stars = [
        (
            32349, "Sirius", 101.287, -16.716, 379.21, -1.46, 0.00, "CMa",
        ),
        (30438, "Canopus", 95.988, -52.696, 10.55, -0.72, 0.15, "Car"),
        (
            71683,
            "Alpha Centauri A",
            219.902,
            -60.834,
            754.81,
            -0.27,
            0.71,
            "Cen",
        ),
        (
            69673, "Arcturus", 213.915, 19.182, 88.83, -0.05, 1.23, "Boo",
        ),
        (91262, "Vega", 279.235, 38.784, 130.23, 0.03, 0.00, "Lyr"),
        (24436, "Capella", 79.172, 45.998, 77.29, 0.08, 0.80, "Aur"),
        (24608, "Rigel", 78.634, -8.202, 3.78, 0.13, -0.03, "Ori"),
        (37279, "Procyon", 114.827, 5.225, 284.56, 0.34, 0.42, "CMi"),
        (27989, "Betelgeuse", 88.793, 7.407, 6.55, 0.42, 1.85, "Ori"),
        (7588, "Achernar", 24.429, -57.237, 22.68, 0.46, -0.16, "Eri"),
        (68702, "Hadar", 210.956, -60.373, 6.21, 0.61, -0.23, "Cen"),
        (97649, "Altair", 297.696, 8.868, 194.45, 0.77, 0.22, "Aql"),
        (60718, "Acrux", 186.650, -63.099, 10.17, 0.76, -0.24, "Cru"),
        (21421, "Aldebaran", 68.980, 16.509, 48.94, 0.85, 1.54, "Tau"),
        (65474, "Spica", 201.298, -11.161, 13.06, 0.97, -0.23, "Vir"),
        (80763, "Antares", 247.352, -26.432, 5.89, 1.06, 1.83, "Sco"),
        (37826, "Pollux", 116.329, 28.026, 96.74, 1.14, 1.00, "Gem"),
        (
            62434,
            "Fomalhaut",
            344.413,
            -29.622,
            129.81,
            1.16,
            0.09,
            "PsA",
        ),
        (25336, "Bellatrix", 81.283, 6.350, 12.92, 1.64, -0.22, "Ori"),
        (26311, "Alnath", 81.573, 28.608, 24.36, 1.65, -0.13, "Tau"),
        (25930, "Mintaka", 83.002, -0.299, 4.71, 2.23, -0.22, "Ori"),
        (26727, "Alnilam", 84.053, -1.202, 1.65, 1.69, -0.18, "Ori"),
        (27366, "Alnitak", 85.190, -1.943, 3.99, 1.77, -0.21, "Ori"),
        (
            113368,
            "Fomalhaut",
            344.413,
            -29.622,
            129.81,
            1.16,
            0.09,
            "PsA",
        ),
        (11767, "Polaris", 37.954, 89.264, 7.54, 1.98, 0.60, "UMi"),
        (54061, "Regulus", 152.093, 11.967, 41.13, 1.35, -0.11, "Leo"),
        (102098, "Deneb", 310.358, 45.280, 2.31, 1.25, 0.09, "Cyg"),
        (61084, "Mimosa", 191.930, -59.689, 9.25, 1.25, -0.23, "Cru"),
        (62956, "Alioth", 193.507, 55.960, 39.51, 1.77, -0.02, "UMa"),
        (67301, "Alkaid", 206.885, 49.313, 31.38, 1.86, -0.19, "UMa"),
        (49669, "Dubhe", 165.933, 61.751, 26.38, 1.79, 1.07, "UMa"),
        (53910, "Merak", 165.460, 56.382, 40.90, 2.37, 0.03, "UMa"),
        (59774, "Phecda", 178.458, 53.695, 38.99, 2.44, 0.04, "UMa"),
        (62956, "Megrez", 183.856, 57.033, 40.05, 3.31, 0.08, "UMa"),
        (65378, "Mizar", 200.981, 54.925, 39.36, 2.27, 0.02, "UMa"),
    ];

    for &(hip, name, ra, dec, parallax, mag, bv, con) in &bright_stars {
        let star = Star::from_catalog(hip, name, ra, dec, parallax, mag, bv, con);
        db.hip_index.insert(hip, db.stars.len());
        db.stars.push(star);
    }

    db
}

// ─────────────────────────────────────────────────────────────────────────────────
// CONSTELLATION LINES (for connecting stars)
// ─────────────────────────────────────────────────────────────────────────────────

/// Constellation line segment connecting two stars by HIP ID
#[derive(Clone, Debug)]
pub struct ConstellationLine {
    pub hip_from: u32,
    pub hip_to: u32,
}

/// Orion constellation lines
pub fn orion_lines() -> Vec<ConstellationLine> {
    vec![
        // Belt: Mintaka - Alnilam - Alnitak
        ConstellationLine {
            hip_from: 25930,
            hip_to: 26727,
        },
        ConstellationLine {
            hip_from: 26727,
            hip_to: 27366,
        },
        // Shoulders
        ConstellationLine {
            hip_from: 27989,
            hip_to: 26311,
        }, // Betelgeuse to Bellatrix
        ConstellationLine {
            hip_from: 26311,
            hip_to: 25336,
        },
        // Betelgeuse to belt
        ConstellationLine {
            hip_from: 27989,
            hip_to: 26727,
        },
        // Belt to Rigel
        ConstellationLine {
            hip_from: 25930,
            hip_to: 24608,
        },
        ConstellationLine {
            hip_from: 27366,
            hip_to: 24608,
        },
    ]
}

/// Big Dipper (Ursa Major) lines
pub fn ursa_major_lines() -> Vec<ConstellationLine> {
    vec![
        ConstellationLine {
            hip_from: 49669,
            hip_to: 53910,
        }, // Dubhe - Merak
        ConstellationLine {
            hip_from: 53910,
            hip_to: 59774,
        }, // Merak - Phecda
        ConstellationLine {
            hip_from: 59774,
            hip_to: 62956,
        }, // Phecda - Megrez
        ConstellationLine {
            hip_from: 62956,
            hip_to: 65378,
        }, // Megrez - Mizar
        ConstellationLine {
            hip_from: 65378,
            hip_to: 67301,
        }, // Mizar - Alkaid
        ConstellationLine {
            hip_from: 62956,
            hip_to: 62956,
        }, // Megrez - Alioth
        ConstellationLine {
            hip_from: 49669,
            hip_to: 62956,
        }, // Dubhe - Megrez
    ]
}

// ─────────────────────────────────────────────────────────────────────────────────
// TESTS
// ─────────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinate_conversion() {
        // Sirius: RA 101.287, Dec -16.716, parallax 379.21 mas
        let pos = ra_dec_distance_to_cartesian(101.287, -16.716, 1000.0 / 379.21);

        // Distance should be ~2.64 pc = ~544000 AU
        let distance_au = pos.length();
        assert!(
            (distance_au - 544000.0).abs() < 10000.0,
            "Sirius distance: {} AU",
            distance_au
        );
    }

    #[test]
    fn test_bright_stars() {
        let db = create_bright_stars();
        assert!(db.len() > 30, "Should have at least 30 bright stars");

        // Find Sirius
        let sirius = db.get_by_hip(32349);
        assert!(sirius.is_some(), "Should find Sirius");

        let sirius = sirius.unwrap();
        assert_eq!(sirius.name, "Sirius");
        assert!(sirius.magnitude < 0.0, "Sirius should be very bright");
    }

    #[test]
    fn test_bv_to_rgb() {
        // Blue star (B-V = -0.3)
        let (r, _g, b) = bv_to_rgb(-0.3);
        assert!(b > r, "Blue star should have more blue than red");

        // Red star (B-V = 1.8)
        let (r, _g, b) = bv_to_rgb(1.8);
        assert!(r > b, "Red star should have more red than blue");
    }
}
