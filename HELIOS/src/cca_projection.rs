//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: cca_projection.rs | HELIOS/src/cca_projection.rs
//! PURPOSE: Object-centric camera with multi-scale celestial visualization
//! CREATED: 2025-12-09
//! UPDATED: 2025-12-10 - Full rewrite for object-centric camera
//! LAYER: HELIOS (simulation)
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! # Object-Centric Camera System
//!
//! When you select an object (Sun, planet, star), that object becomes the camera
//! target and stays at screen center. Zoom changes the scale, not the target.
//!
//! Scale ranges (all in AU):
//! - Planet: 10^-4 to 10^-2 (planetary detail)
//! - Inner: 10^-2 to 1 (Mercury to Mars)
//! - Outer: 1 to 100 (full solar system)
//! - Helio: 100 to 1000 (heliosphere, Voyagers)
//! - Near Stars: 1000 to 10^5 (Alpha Centauri)
//! - Orion: 10^5 to 10^8 (Orion at ~1300 ly)
//!
//! ═══════════════════════════════════════════════════════════════════════════════

#![allow(dead_code)]

use dna::world::cca::{Epoch, FrameId};
use glam::DVec3;
use std::f64::consts::PI;

// ─────────────────────────────────────────────────────────────────────────────────
// CONSTANTS
// ─────────────────────────────────────────────────────────────────────────────────

/// 1 light-year in AU
pub const LY_TO_AU: f64 = 63241.077;

/// 1 parsec in AU
pub const PC_TO_AU: f64 = 206264.806;

/// Maximum planets supported
pub const MAX_PLANETS: usize = 16;

// ─────────────────────────────────────────────────────────────────────────────────
// OBJECT IDENTIFICATION
// ─────────────────────────────────────────────────────────────────────────────────

/// Unique identifier for celestial objects
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum ObjectId {
    #[default]
    Sun,
    Planet(usize), // 0=Mercury, 1=Venus, ... 7=Neptune
    Star(u32),     // Hipparcos ID
    Position,      // Free camera at arbitrary position
}

// ─────────────────────────────────────────────────────────────────────────────────
// SCALE LEVELS
// ─────────────────────────────────────────────────────────────────────────────────

/// Scale levels for multi-scale rendering
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaleLevel {
    /// Planetary detail (moons, surface features)
    Planet,
    /// Inner solar system (Mercury to Mars)
    Inner,
    /// Outer solar system (to Neptune)
    Outer,
    /// Heliosphere (to Voyagers)
    Heliosphere,
    /// Nearby stars (Alpha Centauri, Sirius)
    NearStars,
    /// Distant constellations (Orion)
    FarStars,
}

impl ScaleLevel {
    /// Determine scale level from current AU/pixel zoom
    pub fn from_scale(scale: f64) -> Self {
        // scale = AU visible in viewport height
        if scale < 0.01 {
            ScaleLevel::Planet
        } else if scale < 5.0 {
            ScaleLevel::Inner
        } else if scale < 100.0 {
            ScaleLevel::Outer
        } else if scale < 1000.0 {
            ScaleLevel::Heliosphere
        } else if scale < 100_000.0 {
            ScaleLevel::NearStars
        } else {
            ScaleLevel::FarStars
        }
    }

    /// Get the maximum magnitude of stars to render at this scale
    pub fn star_magnitude_limit(&self) -> f64 {
        // FIXED: Stars should be visible at ALL zoom levels
        // Close to Earth = night sky view = show all visible stars
        // Far away = deep space = still show stars
        match self {
            ScaleLevel::Planet => 6.0,      // Full night sky (naked eye limit)
            ScaleLevel::Inner => 6.0,       // Full night sky
            ScaleLevel::Outer => 5.0,       // Bright night sky
            ScaleLevel::Heliosphere => 5.0, // Bright stars
            ScaleLevel::NearStars => 6.0,   // Naked eye limit
            ScaleLevel::FarStars => 8.0,    // Telescope limit
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────────
// CELESTIAL CAMERA
// ─────────────────────────────────────────────────────────────────────────────────

/// Object-centric camera that keeps the target at screen center
#[derive(Clone, Debug)]
pub struct CelestialCamera {
    // ── Target ───────────────────────────────────────────────────────────────────
    /// What object we're looking at
    pub target: ObjectId,
    /// Target's current position in HCI frame (AU) - updated each frame
    pub target_position: DVec3,

    // ── Camera orientation ───────────────────────────────────────────────────────
    /// Azimuth angle (rotation around Z axis) in radians
    pub azimuth: f64,
    /// Elevation angle (tilt up from XY plane) in radians [0, π*0.45]
    pub elevation: f64,

    // ── Scale ────────────────────────────────────────────────────────────────────
    /// AU visible in viewport height (orthographic scale)
    /// Range: ~10^-6 (planetary surface) to ~10^8 (Orion distance)
    pub scale: f64,
    /// Current scale level for LOD rendering
    pub scale_level: ScaleLevel,

    // ── Viewport ─────────────────────────────────────────────────────────────────
    pub viewport_width: f64,
    pub viewport_height: f64,

    // ── Frame system ─────────────────────────────────────────────────────────────
    /// Current reference frame (FrameGraph stored separately due to closures)
    pub current_frame: FrameId,
    /// Current epoch for time-dependent transforms
    pub current_epoch: Epoch,
}

impl Default for CelestialCamera {
    fn default() -> Self {
        Self {
            target: ObjectId::Sun,
            target_position: DVec3::ZERO,
            azimuth: 0.0,
            elevation: PI * 0.15, // 27 degrees above plane
            scale: 5.0,           // Inner solar system view
            scale_level: ScaleLevel::Inner,
            viewport_width: 1920.0,
            viewport_height: 1080.0,
            current_frame: FrameId::HCI,
            current_epoch: Epoch::j2000(),
        }
    }
}

impl CelestialCamera {
    /// Create camera looking at Sun with given viewport
    pub fn new(width: f64, height: f64) -> Self {
        Self {
            viewport_width: width,
            viewport_height: height,
            ..Default::default()
        }
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // TARGET MANAGEMENT
    // ═════════════════════════════════════════════════════════════════════════════

    /// Set camera target to an object
    pub fn set_target(&mut self, target: ObjectId) {
        self.target = target;
        // Note: target_position must be updated by SimulationState each frame
    }

    /// Update target position from simulation state
    /// Called each frame with the current position of the target object
    pub fn update_target_position(&mut self, position: DVec3) {
        self.target_position = position;
    }

    /// Set camera to look at Sun
    pub fn focus_sun(&mut self) {
        self.target = ObjectId::Sun;
        self.target_position = DVec3::ZERO;
        self.scale = 0.01; // Close-up view
        self.scale_level = ScaleLevel::Planet;
    }

    /// Set camera to look at a planet
    pub fn focus_planet(&mut self, idx: usize, position: DVec3, radius_au: f64) {
        self.target = ObjectId::Planet(idx);
        self.target_position = position;
        // Scale so planet fills about 1/3 of screen height
        self.scale = (radius_au * 6.0).clamp(0.001, 5.0);
        self.scale_level = ScaleLevel::from_scale(self.scale);
    }

    /// Set camera to free position mode (for panning away from objects)
    pub fn set_free_position(&mut self, position: DVec3) {
        self.target = ObjectId::Position;
        self.target_position = position;
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // SCALE CONTROL
    // ═════════════════════════════════════════════════════════════════════════════

    /// Set scale directly (AU visible in viewport height)
    pub fn set_scale(&mut self, scale: f64) {
        // Clamp to valid range: planetary detail to Orion distance
        self.scale = scale.clamp(1e-6, 1e8);
        self.scale_level = ScaleLevel::from_scale(self.scale);
    }

    /// Zoom by factor (< 1 = zoom in, > 1 = zoom out)
    pub fn zoom_by(&mut self, factor: f64) {
        self.set_scale(self.scale * factor);
    }

    /// Get AU per pixel at current scale
    pub fn au_per_pixel(&self) -> f64 {
        self.scale / self.viewport_height
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // CAMERA ROTATION
    // ═════════════════════════════════════════════════════════════════════════════

    /// Set camera angles directly
    pub fn set_angles(&mut self, azimuth: f64, elevation: f64) {
        self.azimuth = azimuth;
        self.elevation = elevation.clamp(0.0, PI * 0.45);
    }

    /// Orbit camera by delta angles
    pub fn orbit(&mut self, d_azimuth: f64, d_elevation: f64) {
        self.azimuth += d_azimuth;
        self.elevation = (self.elevation + d_elevation).clamp(0.0, PI * 0.45);
    }

    /// Pan camera (moves target position in screen plane)
    /// Only works when target is ObjectId::Position
    pub fn pan(&mut self, dx_screen: f64, dy_screen: f64) {
        if self.target != ObjectId::Position {
            // First convert to free position mode
            self.target = ObjectId::Position;
        }

        // Convert screen delta to world delta
        let au_per_pixel = self.au_per_pixel();

        // Get camera right and up vectors
        let (right, up) = self.camera_basis_vectors();

        // Move target in screen plane
        self.target_position += right * (-dx_screen * au_per_pixel);
        self.target_position += up * (dy_screen * au_per_pixel);
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // VIEWPORT
    // ═════════════════════════════════════════════════════════════════════════════

    /// Update viewport dimensions
    pub fn set_viewport(&mut self, width: f64, height: f64) {
        self.viewport_width = width;
        self.viewport_height = height;
    }

    /// Get aspect ratio
    pub fn aspect(&self) -> f64 {
        self.viewport_width / self.viewport_height
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // EPOCH
    // ═════════════════════════════════════════════════════════════════════════════

    /// Set current epoch from Julian Date
    pub fn set_epoch_jd(&mut self, jd: f64) {
        self.current_epoch = Epoch::from_jd(jd, dna::world::cca::epoch::TimeScale::TDB);
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // PROJECTION
    // ═════════════════════════════════════════════════════════════════════════════

    /// Get camera basis vectors (right, up) in world space
    fn camera_basis_vectors(&self) -> (DVec3, DVec3) {
        // Camera looks from position toward target
        // We compute the viewing direction from azimuth/elevation
        let cos_elev = self.elevation.cos();
        let sin_elev = self.elevation.sin();
        let cos_az = self.azimuth.cos();
        let sin_az = self.azimuth.sin();

        // Forward direction (from camera to target)
        let forward = DVec3::new(cos_elev * sin_az, -cos_elev * cos_az, sin_elev);

        // Right vector (perpendicular to forward and Z-up)
        let world_up = DVec3::Z;
        let right = forward.cross(world_up).normalize();

        // Correct up vector (perpendicular to forward and right)
        let up = right.cross(forward).normalize();

        (right, up)
    }

    /// Project a world position (in AU, HCI frame) to screen coordinates
    ///
    /// Returns (screen_x, screen_y, depth) where:
    /// - screen_x, screen_y: pixel coordinates (0,0 = top-left)
    /// - depth: distance from camera plane (for sorting, larger = further)
    pub fn project(&self, world_pos: DVec3) -> (f64, f64, f64) {
        // 1. Position relative to target (prevents floating-point precision loss)
        let rel_pos = world_pos - self.target_position;

        // 2. Get camera basis vectors
        let (right, up) = self.camera_basis_vectors();

        // Forward direction
        let cos_elev = self.elevation.cos();
        let sin_elev = self.elevation.sin();
        let cos_az = self.azimuth.cos();
        let sin_az = self.azimuth.sin();
        let forward = DVec3::new(cos_elev * sin_az, -cos_elev * cos_az, sin_elev);

        // 3. Transform to camera space
        let cam_x = rel_pos.dot(right); // Right is X
        let cam_y = rel_pos.dot(up); // Up is Y
        let cam_z = rel_pos.dot(forward); // Forward is Z (depth)

        // 4. Orthographic projection (no perspective distortion)
        let half_height = self.scale / 2.0;
        let half_width = half_height * self.aspect();

        let ndc_x = cam_x / half_width;
        let ndc_y = cam_y / half_height;

        // 5. NDC to screen (flip Y for canvas coordinates)
        let screen_x = (ndc_x + 1.0) * 0.5 * self.viewport_width;
        let screen_y = (1.0 - ndc_y) * 0.5 * self.viewport_height;

        (screen_x, screen_y, cam_z)
    }

    // Note: project_from_frame() removed - FrameGraph stored in SimulationState
    // Use SimulationState::project_from_frame() instead

    /// Check if a point is visible on screen
    pub fn is_visible(&self, world_pos: DVec3) -> bool {
        let (x, y, _depth) = self.project(world_pos);
        x >= -100.0
            && x <= self.viewport_width + 100.0  // Small margin for large objects
            && y >= -100.0
            && y <= self.viewport_height + 100.0
    }

    /// Convert screen position to world AU (in the orbital plane z=0)
    pub fn screen_to_au(&self, screen_x: f64, screen_y: f64) -> DVec3 {
        // Inverse of projection (for z=0 plane)
        let ndc_x = (screen_x / self.viewport_width) * 2.0 - 1.0;
        let ndc_y = 1.0 - (screen_y / self.viewport_height) * 2.0;

        let half_height = self.scale / 2.0;
        let half_width = half_height * self.aspect();

        let cam_x = ndc_x * half_width;
        let cam_y = ndc_y * half_height;

        // Get basis vectors
        let (right, up) = self.camera_basis_vectors();

        // Transform back to world (approximate, assumes z=0 plane)
        self.target_position + right * cam_x + up * cam_y
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // PRESET VIEWS
    // ═════════════════════════════════════════════════════════════════════════════

    /// Reset to inner solar system view
    pub fn view_inner_system(&mut self) {
        self.target = ObjectId::Sun;
        self.target_position = DVec3::ZERO;
        self.scale = 3.5; // Mercury to Mars visible
        self.scale_level = ScaleLevel::Inner;
        self.azimuth = 0.0;
        self.elevation = PI * 0.15;
    }

    /// Reset to outer solar system view
    pub fn view_outer_system(&mut self) {
        self.target = ObjectId::Sun;
        self.target_position = DVec3::ZERO;
        self.scale = 100.0; // All planets visible
        self.scale_level = ScaleLevel::Outer;
    }

    /// View heliosphere
    pub fn view_heliosphere(&mut self) {
        self.target = ObjectId::Sun;
        self.target_position = DVec3::ZERO;
        self.scale = 400.0; // To heliopause
        self.scale_level = ScaleLevel::Heliosphere;
    }

    /// View to nearby stars (Alpha Centauri visible)
    pub fn view_nearby_stars(&mut self) {
        self.target = ObjectId::Sun;
        self.target_position = DVec3::ZERO;
        self.scale = 5.0 * LY_TO_AU; // ~5 light years
        self.scale_level = ScaleLevel::NearStars;
    }

    /// View to Orion constellation
    pub fn view_orion(&mut self) {
        self.target = ObjectId::Sun;
        self.target_position = DVec3::ZERO;
        self.scale = 2000.0 * LY_TO_AU; // ~2000 light years
        self.scale_level = ScaleLevel::FarStars;
    }
}

// ─────────────────────────────────────────────────────────────────────────────────
// COORDINATE DISPLAY UTILITIES
// ─────────────────────────────────────────────────────────────────────────────────

/// Format a scale value for human display
pub fn format_scale(scale_au: f64) -> String {
    if scale_au < 0.001 {
        format!("{:.0} km", scale_au * 149_597_870.7)
    } else if scale_au < 1.0 {
        format!("{:.3} AU", scale_au)
    } else if scale_au < 1000.0 {
        format!("{:.1} AU", scale_au)
    } else if scale_au < LY_TO_AU * 10.0 {
        format!("{:.2} ly", scale_au / LY_TO_AU)
    } else {
        format!("{:.0} ly", scale_au / LY_TO_AU)
    }
}

/// Format position for display
pub fn format_position(pos: DVec3, frame: FrameId) -> String {
    let frame_name = match frame {
        FrameId::HCI => "HCI",
        FrameId::HEE => "HEE",
        FrameId::GCI => "GCI",
        FrameId::GSE => "GSE",
        _ => "???",
    };

    format!(
        "{} ({:.3}, {:.3}, {:.3}) AU",
        frame_name, pos.x, pos.y, pos.z
    )
}

/// Convert AU to other units
pub fn au_to_km(au: f64) -> f64 {
    au * 149_597_870.7
}

pub fn au_to_solar_radii(au: f64) -> f64 {
    au * 149_597_870.7 / 695_700.0
}

pub fn au_to_light_years(au: f64) -> f64 {
    au / LY_TO_AU
}

pub fn au_to_parsecs(au: f64) -> f64 {
    au / PC_TO_AU
}

// ─────────────────────────────────────────────────────────────────────────────────
// TESTS
// ─────────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_default() {
        let camera = CelestialCamera::default();
        assert_eq!(camera.target, ObjectId::Sun);
        assert_eq!(camera.target_position, DVec3::ZERO);
    }

    #[test]
    fn test_project_origin() {
        let camera = CelestialCamera::new(1920.0, 1080.0);
        // Sun at origin should project to screen center
        let (x, y, _depth) = camera.project(DVec3::ZERO);
        assert!((x - 960.0).abs() < 1.0, "x={}, expected ~960", x);
        assert!((y - 540.0).abs() < 1.0, "y={}, expected ~540", y);
    }

    #[test]
    fn test_zoom() {
        let mut camera = CelestialCamera::default();
        let initial_scale = camera.scale;

        camera.zoom_by(2.0);
        assert!((camera.scale - initial_scale * 2.0).abs() < 0.001);

        camera.zoom_by(0.5);
        assert!((camera.scale - initial_scale).abs() < 0.001);
    }

    #[test]
    fn test_scale_levels() {
        assert_eq!(ScaleLevel::from_scale(0.001), ScaleLevel::Planet);
        assert_eq!(ScaleLevel::from_scale(1.0), ScaleLevel::Inner);
        assert_eq!(ScaleLevel::from_scale(50.0), ScaleLevel::Outer);
        assert_eq!(ScaleLevel::from_scale(500.0), ScaleLevel::Heliosphere);
        assert_eq!(ScaleLevel::from_scale(50_000.0), ScaleLevel::NearStars);
        assert_eq!(ScaleLevel::from_scale(1_000_000.0), ScaleLevel::FarStars);
    }

    #[test]
    fn test_target_at_center() {
        let mut camera = CelestialCamera::new(1920.0, 1080.0);
        camera.scale = 10.0;

        // Move target to (5, 5, 0)
        camera.target_position = DVec3::new(5.0, 5.0, 0.0);

        // Target should project to screen center
        let (x, y, _) = camera.project(DVec3::new(5.0, 5.0, 0.0));
        assert!((x - 960.0).abs() < 1.0, "target x={}, expected ~960", x);
        assert!((y - 540.0).abs() < 1.0, "target y={}, expected ~540", y);
    }

    #[test]
    fn test_format_scale() {
        assert!(format_scale(0.0001).contains("km"));
        assert!(format_scale(0.5).contains("AU"));
        assert!(format_scale(100.0).contains("AU"));
        assert!(format_scale(LY_TO_AU * 5.0).contains("ly"));
    }
}
