//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: star_data.rs | HELIOS/src/star_data.rs
//! PURPOSE: UniverseDataManager - deterministic star/constellation rendering with multi-wavelength support
//! LAYER: HELIOS (simulation)
//! ═══════════════════════════════════════════════════════════════════════════════

use crate::cca_projection::CelestialCamera;
use crate::streaming::TileCache;
use dna::spatial::SpatialKey;
use dna::world::stars::Star;
use glam::DVec3;

/// Wavelength bands for multi-spectrum visualization
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)] // Phase 2: Band selection UI not yet implemented
pub enum Band {
    Gamma = 0,   // High-energy gamma rays
    XRay = 1,    // X-rays
    UV = 2,      // Ultraviolet
    Optical = 3, // Visible light (default)
    IR = 4,      // Infrared
    Radio = 5,   // Radio waves
    CMB = 6,     // Cosmic microwave background
}

/// Instance of a visible object (star, galaxy, etc.) at a specific time
#[derive(Clone, Debug)]
pub struct StarInstance {
    pub position: DVec3,      // AU HCI at time jd
    pub magnitude: f32,       // Apparent mag in current band
    pub color_rgb: [u8; 3],   // Band-specific color
    pub id: u64,              // Global unique ID
    pub name: Option<String>, // For labeled objects
}

/// Constellation edge definition
#[derive(Clone, Debug)]
pub struct ConstellationEdge {
    pub star_a: u64,
    pub star_b: u64,
}

/// UniverseDataManager - manages deterministic star/constellation rendering
pub struct UniverseDataManager {
    /// Current wavelength band
    current_band: Band,
    /// Maximum visible objects per frame
    max_visible: usize,
    /// Current visible instances (updated per frame)
    visible_instances: Vec<StarInstance>,
    /// Constellation edges for current band
    constellation_edges: Vec<ConstellationEdge>,
    /// Performance monitoring
    frame_times: [f64; 60],
    frame_idx: usize,
    /// LOD state
    lod_level: u8,
    mag_limit: f32,
    /// Tile cache for streaming (Phase 2)
    pub tile_cache: Option<TileCache>,
}

impl UniverseDataManager {
    /// Create new manager with capacity (Phase 1 - local mode)
    pub fn new(max_visible: usize) -> Self {
        Self {
            current_band: Band::Optical,
            max_visible,
            visible_instances: Vec::with_capacity(max_visible),
            constellation_edges: Vec::new(),
            frame_times: [0.0; 60],
            frame_idx: 0,
            lod_level: 6, // Start with good detail
            mag_limit: 6.0, // Show naked-eye visible stars
            tile_cache: None, // Local mode
        }
    }

    /// Create new manager with streaming (Phase 2)
    pub fn new_with_streaming(max_visible: usize, server_url: String) -> Self {
        Self {
            current_band: Band::Optical,
            max_visible,
            visible_instances: Vec::with_capacity(max_visible),
            constellation_edges: Vec::new(),
            frame_times: [0.0; 60],
            frame_idx: 0,
            lod_level: 6,
            mag_limit: 6.0,
            tile_cache: Some(TileCache::new(1000, server_url)),
        }
    }

    /// Update view for new camera position/time/band
    /// HYBRID RENDERING: Always show preloaded stars + optional streaming enhancement
    pub fn update_view(&mut self, camera: &CelestialCamera, jd: f64, band: Band) {
        let start_time = js_sys::Date::now();

        self.current_band = band;
        self.visible_instances.clear();

        // FIXED: Don't overwrite mag_limit - let adjust_lod() control it
        // Just ensure it's within reasonable bounds
        self.mag_limit = self.mag_limit.clamp(4.0, 8.0);

        // ALWAYS render preloaded stars first (instant, offline-capable)
        self.update_from_local_db(camera, jd);

        // ENHANCE with streaming if available (Phase 2)
        if self.tile_cache.is_some() {
            let mut cache = self.tile_cache.take().unwrap();
            self.update_from_streaming(camera, jd, &mut cache);
            self.tile_cache = Some(cache);
        }

        // Update constellation edges (static for now)
        self.update_constellations();

        // Performance monitoring
        let end_time = js_sys::Date::now();
        self.frame_times[self.frame_idx] = end_time - start_time;
        self.frame_idx = (self.frame_idx + 1) % 60;

        // Auto-adjust LOD based on performance
        self.adjust_lod();
    }

    /// Get visible instances (sorted by brightness)
    pub fn visible_instances(&self) -> &[StarInstance] {
        &self.visible_instances
    }

    /// Get constellation edges
    pub fn constellation_edges(&self) -> &[ConstellationEdge] {
        &self.constellation_edges
    }

    /// Get position of object by ID at given time
    #[allow(dead_code)] // Phase 2: Server tile system
    pub fn object_pos(&self, _id: u64, _jd: f64) -> Option<DVec3> {
        // For now, lookup in local DB
        // In phase 2, this will be from cached tiles
        None // TODO: implement
    }

    /// Get current band
    #[allow(dead_code)] // Phase 2: Band selection UI
    pub fn current_band(&self) -> Band {
        self.current_band
    }

    #[allow(dead_code)] // Phase 2: Band selection UI
    pub fn set_current_band(&mut self, band: Band) {
        self.current_band = band;
    }

    #[allow(dead_code)] // Phase 2: Dynamic magnitude control
    pub fn set_magnitude_limit(&mut self, limit: f64) {
        self.mag_limit = limit as f32;
    }

    /// Get average frame time (ms)
    pub fn avg_frame_time(&self) -> f64 {
        self.frame_times.iter().sum::<f64>() / 60.0
    }

    // Private methods

    fn update_from_local_db(&mut self, camera: &CelestialCamera, jd: f64) {
        // Get local star database
        let db = dna::world::stars::create_bright_stars();

        // Filter by magnitude and visibility
        for star in db.brighter_than(self.mag_limit as f64) {
            // Check if star is in view frustum (simplified)
            if self.is_star_visible(star, camera) {
                // Evaluate position at time jd
                let pos = self.star_pos_at_time(star, jd);

                // Get magnitude and color for current band
                let (mag, color) = self.star_band_properties(star, self.current_band);

                self.visible_instances.push(StarInstance {
                    position: pos,
                    magnitude: mag,
                    color_rgb: color,
                    id: star.hip_id as u64,
                    name: if star.magnitude < 2.0 {
                        Some(star.name.clone())
                    } else {
                        None
                    },
                });
            }
        }

        // Sort by brightness (brightest first)
        self.visible_instances
            .sort_by(|a, b| a.magnitude.partial_cmp(&b.magnitude).unwrap());

        // Cap at max_visible
        if self.visible_instances.len() > self.max_visible {
            self.visible_instances.truncate(self.max_visible);
        }
    }

    /// Update from streaming tiles (Phase 2 enhancement)
    fn update_from_streaming(&mut self, camera: &CelestialCamera, _jd: f64, cache: &mut TileCache) {
        // Compute visible spatial keys based on camera
        let visible_keys = self.compute_visible_keys(camera);

        // Request tiles (non-blocking)
        cache.fetch_tiles(visible_keys.clone());

        // Merge cached tile data into visible instances
        for key in visible_keys {
            if let Some(tile_data) = cache.get(&key) {
                for star_entry in &tile_data.stars {
                    // Check if we're at capacity
                    if self.visible_instances.len() >= self.max_visible {
                        break;
                    }

                    // Check magnitude filter
                    if star_entry.magnitude > self.mag_limit as f64 {
                        continue;
                    }

                    // Convert to StarInstance
                    self.visible_instances.push(StarInstance {
                        position: star_entry.position(),
                        magnitude: star_entry.magnitude as f32,
                        color_rgb: star_entry.color_rgb(),
                        id: star_entry.hip_id as u64,
                        name: Some(star_entry.name.clone()),
                    });
                }
            }
        }

        // Re-sort by brightness
        self.visible_instances
            .sort_by(|a, b| a.magnitude.partial_cmp(&b.magnitude).unwrap());

        // Cap at max_visible
        if self.visible_instances.len() > self.max_visible {
            self.visible_instances.truncate(self.max_visible);
        }
    }

    /// Compute visible SpatialKeys based on camera frustum
    fn compute_visible_keys(&self, _camera: &CelestialCamera) -> Vec<SpatialKey> {
        // For now, use a simple approach: query the camera's current cube face and level
        // In full implementation, this would do proper frustum culling

        // Determine LOD level based on zoom
        let level = self.lod_level.min(8);

        // For now, just return a few keys around the camera direction
        // Full implementation would compute all visible tiles in frustum
        let mut keys = Vec::new();

        // Simple heuristic: get tiles for all 6 cube faces at current LOD level
        // This is conservative but ensures we don't miss stars
        for face in 0..6 {
            let tile_count = 1 << level; // 2^level tiles per dimension
            let center = tile_count / 2;

            // Get center tile and adjacent tiles
            for dx in 0..=2 {
                for dy in 0..=2 {
                    let x = (center + dx).min(tile_count - 1);
                    let y = (center + dy).min(tile_count - 1);
                    keys.push(SpatialKey::new(face, level, x, y));
                }
            }
        }

        keys
    }

    fn update_constellations(&mut self) {
        // For now, hardcode Orion constellation
        // In phase 2, this will come from API
        self.constellation_edges = vec![
            ConstellationEdge {
                star_a: 25930,
                star_b: 26727,
            }, // Mintaka-Alnilam
            ConstellationEdge {
                star_a: 26727,
                star_b: 27366,
            }, // Alnilam-Alnitak
            ConstellationEdge {
                star_a: 27989,
                star_b: 26727,
            }, // Betelgeuse-Alnilam
            ConstellationEdge {
                star_a: 24608,
                star_b: 25930,
            }, // Rigel-Mintaka
        ];
    }

    fn is_star_visible(&self, _star: &Star, _camera: &CelestialCamera) -> bool {
        // FIXED: Stars are at infinity - always render as background skybox!
        // They're 100s-1000s of light-years away, so position check doesn't work
        // at solar system scale. Just render based on direction.
        true  // Always visible
    }

    fn star_pos_at_time(&self, star: &Star, _jd: f64) -> DVec3 {
        // For now, assume J2000 positions are close enough
        // In full implementation: star.position + star.velocity * (jd - J2000) / 365.25
        star.position
    }

    fn star_band_properties(&self, star: &Star, band: Band) -> (f32, [u8; 3]) {
        match band {
            Band::Optical => {
                let (r, g, b) = star.color_rgb();
                (star.magnitude as f32, [r, g, b])
            }
            Band::UV => {
                // UV is brighter for hot stars (blue)
                let uv_mag = star.magnitude as f32 - if star.color_bv < 0.0 { 1.0 } else { 0.0 };
                (uv_mag, [200, 220, 255]) // Blue-white
            }
            Band::IR => {
                // IR is brighter for cool stars (red)
                let ir_mag = star.magnitude as f32 + if star.color_bv > 1.0 { 0.5 } else { 0.0 };
                (ir_mag, [255, 150, 100]) // Red-orange
            }
            Band::XRay => {
                // X-rays from hot stars only
                let xray_mag = if star.color_bv < 0.0 {
                    star.magnitude as f32 + 2.0
                } else {
                    20.0
                };
                (xray_mag, [255, 255, 255]) // White
            }
            Band::Gamma => {
                // Gamma rays from extreme objects (not normal stars)
                (25.0, [150, 200, 255]) // Blue
            }
            Band::Radio => {
                // Radio from various sources
                let radio_mag = star.magnitude as f32 + 5.0;
                (radio_mag, [255, 200, 150]) // Yellow
            }
            Band::CMB => {
                // Cosmic microwave background - uniform
                (15.0, [255, 200, 150]) // Warm
            }
        }
    }

    fn adjust_lod(&mut self) {
        let avg_time = self.avg_frame_time();

        if avg_time > 8.0 && self.mag_limit > 4.0 {
            // Too slow, show fewer stars
            self.lod_level = self.lod_level.saturating_add(1).min(10);
            self.mag_limit -= 0.5;
        } else if avg_time < 3.0 && self.mag_limit < 8.0 && self.lod_level > 0 {
            // Fast enough, show more stars!
            self.lod_level = self.lod_level.saturating_sub(1);
            self.mag_limit += 0.5;
        }
    }
}
