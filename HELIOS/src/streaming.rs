//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: streaming.rs | HELIOS/src/streaming.rs
//! PURPOSE: Client-side tile streaming with LRU cache (Phase 2)
//! LAYER: HELIOS (WASM client)
//! ═══════════════════════════════════════════════════════════════════════════════

use dna::spatial::SpatialKey;
use glam::DVec3;
use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::num::NonZeroUsize;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;

/// Compact star entry from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarEntry {
    pub hip_id: i32,
    pub name: String,
    pub ra: f64,
    pub dec: f64,
    pub magnitude: f64,
    pub color_bv: f64,
    pub constellation: String,
}

impl StarEntry {
    /// Convert RA/Dec to Cartesian position in HCI frame
    pub fn position(&self) -> DVec3 {
        let distance_pc = 100.0; // Default distance for stars without parallax
        let distance_au = distance_pc * 206264.806;

        let ra_rad = self.ra.to_radians();
        let dec_rad = self.dec.to_radians();

        DVec3::new(
            distance_au * dec_rad.cos() * ra_rad.cos(),
            distance_au * dec_rad.cos() * ra_rad.sin(),
            distance_au * dec_rad.sin(),
        )
    }

    /// Get RGB color from B-V index
    pub fn color_rgb(&self) -> [u8; 3] {
        // Same as DNA star color mapping
        let temp = 4600.0 * (1.0 / (0.92 * self.color_bv + 1.7) + 1.0 / (0.92 * self.color_bv + 0.62));

        let r = if temp <= 6600.0 {
            255.0
        } else {
            329.698727446 * ((temp - 6000.0) / 100.0).powf(-0.1332047592)
        };

        let g = if temp <= 6600.0 {
            99.4708025861 * ((temp / 100.0).ln()) - 161.1195681661
        } else {
            288.1221695283 * ((temp - 6000.0) / 100.0).powf(-0.0755148492)
        };

        let b = if temp >= 6600.0 {
            255.0
        } else if temp <= 1900.0 {
            0.0
        } else {
            138.5177312231 * (((temp - 1000.0) / 100.0).ln()) - 305.0447927307
        };

        [
            r.clamp(0.0, 255.0) as u8,
            g.clamp(0.0, 255.0) as u8,
            b.clamp(0.0, 255.0) as u8,
        ]
    }
}

/// Tile data from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileData {
    pub face: u8,
    pub level: u8,
    pub x: u32,
    pub y: u32,
    pub stars: Vec<StarEntry>,
}

/// LRU cache for tile streaming
pub struct TileCache {
    cache: LruCache<SpatialKey, TileData>,
    pending: HashSet<SpatialKey>,
    server_url: String,
}

impl TileCache {
    /// Create new tile cache with capacity
    pub fn new(capacity: usize, server_url: String) -> Self {
        console::log_1(&format!("TileCache initialized: capacity={}, server={}", capacity, server_url).into());

        Self {
            cache: LruCache::new(NonZeroUsize::new(capacity).unwrap()),
            pending: HashSet::new(),
            server_url,
        }
    }

    /// Get tile from cache (if loaded)
    pub fn get(&mut self, key: &SpatialKey) -> Option<&TileData> {
        self.cache.get(key)
    }

    /// Check if tile is cached or pending
    pub fn has(&self, key: &SpatialKey) -> bool {
        self.cache.contains(key) || self.pending.contains(key)
    }

    /// Fetch tiles for given spatial keys
    pub fn fetch_tiles(&mut self, keys: Vec<SpatialKey>) {
        for key in keys {
            if !self.has(&key) {
                self.spawn_fetch(key);
            }
        }
    }

    /// Spawn async fetch task for a tile
    fn spawn_fetch(&mut self, key: SpatialKey) {
        self.pending.insert(key);

        let url = format!(
            "{}/api/tiles/stars/{}/{}/{}/{}",
            self.server_url,
            key.face(),
            key.level(),
            key.coords().0,
            key.coords().1
        );

        console::log_1(&format!("Fetching tile: face={} level={} coords={:?}",
            key.face(), key.level(), key.coords()).into());

        spawn_local(async move {
            match fetch_tile_data(&url).await {
                Ok(tile_data) => {
                    console::log_1(&format!(
                        "Loaded tile: {} stars from face={} level={} coords={:?}",
                        tile_data.stars.len(),
                        tile_data.face,
                        tile_data.level,
                        (tile_data.x, tile_data.y)
                    ).into());

                    // Store in cache (requires message passing for thread safety in WASM)
                    // For now, this is a placeholder - proper implementation needs
                    // a Rc<RefCell<TileCache>> or message channel
                },
                Err(e) => {
                    console::error_1(&format!("Failed to fetch tile: {}", e).into());
                }
            }
        });
    }

    /// Get all cached tiles
    pub fn iter(&mut self) -> impl Iterator<Item = (&SpatialKey, &TileData)> {
        self.cache.iter()
    }
}

/// Fetch tile data from server (async)
async fn fetch_tile_data(url: &str) -> Result<TileData, String> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }

    let tile_data: TileData = response
        .json()
        .await
        .map_err(|e| format!("JSON parse error: {}", e))?;

    Ok(tile_data)
}
