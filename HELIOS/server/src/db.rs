//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: db.rs | HELIOS/server/src/db.rs
//! PURPOSE: Database queries for star data based on SpatialKey
//! LAYER: HELIOS Server
//! ═══════════════════════════════════════════════════════════════════════════════

use anyhow::Result;
use dna::spatial::SpatialKey;
use glam::DVec3;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;

/// Star data from database (matches DB schema)
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StarData {
    pub hip_id: i32,
    pub name: String,
    pub ra: f64,           // Right Ascension (degrees)
    pub dec: f64,          // Declination (degrees)
    pub parallax: f64,     // Milliarcseconds
    pub magnitude: f64,
    pub color_bv: f64,
    pub constellation: String,
}

impl StarData {
    /// Convert RA/Dec to Cartesian position in HCI frame
    pub fn position(&self) -> DVec3 {
        let distance_pc = if self.parallax > 0.0 {
            1000.0 / self.parallax
        } else {
            1000.0 // Default to 1kpc for distant stars
        };

        let distance_au = distance_pc * 206264.806;

        let ra_rad = self.ra.to_radians();
        let dec_rad = self.dec.to_radians();

        DVec3::new(
            distance_au * dec_rad.cos() * ra_rad.cos(),
            distance_au * dec_rad.cos() * ra_rad.sin(),
            distance_au * dec_rad.sin(),
        )
    }
}

/// Star database query manager
pub struct StarDatabase {
    pool: PgPool,
}

impl StarDatabase {
    /// Connect to PostgreSQL database
    pub async fn new(db_url: &str) -> Result<Self> {
        tracing::info!("Connecting to database: {}", db_url);
        let pool = PgPool::connect(db_url).await?;
        Ok(Self { pool })
    }

    /// Query stars within a spatial tile
    pub async fn query_tile(&self, key: SpatialKey) -> Result<Vec<StarData>> {
        let level = key.level();

        // Get tile direction vector
        let direction = key.direction();
        let (ra, dec) = cartesian_to_ra_dec(direction);

        // Angular radius for this LOD level (degrees)
        let angular_radius = angular_size_for_level(level);

        // Magnitude limit for this LOD level
        let mag_limit = magnitude_limit_for_level(level);

        tracing::debug!(
            "Query tile: face={}, level={}, coords={:?}, center=({:.2}, {:.2}), radius={:.2}°, mag<{:.1}",
            key.face(),
            level,
            key.coords(),
            ra,
            dec,
            angular_radius,
            mag_limit
        );

        // Query stars within angular distance from tile center
        let stars = sqlx::query_as::<_, StarData>(
            r#"
            SELECT hip_id, name, ra, dec, parallax, magnitude, color_bv, constellation
            FROM stars
            WHERE magnitude < $1
              AND (
                  -- Simple angular distance approximation (works for small angles)
                  POW(ra - $2, 2) + POW(dec - $3, 2) < POW($4, 2)
              )
            ORDER BY magnitude ASC
            LIMIT 1000
            "#,
        )
        .bind(mag_limit)
        .bind(ra)
        .bind(dec)
        .bind(angular_radius)
        .fetch_all(&self.pool)
        .await?;

        tracing::debug!("Found {} stars in tile", stars.len());
        Ok(stars)
    }

    /// Batch query for multiple tiles (optimization)
    pub async fn query_tiles_batch(&self, keys: Vec<SpatialKey>) -> Result<HashMap<SpatialKey, Vec<StarData>>> {
        let mut results = HashMap::new();

        // Execute queries in parallel
        let futures: Vec<_> = keys.iter()
            .map(|&k| async move {
                let stars = self.query_tile(k).await?;
                Ok::<(SpatialKey, Vec<StarData>), anyhow::Error>((k, stars))
            })
            .collect();

        let batch_results = futures::future::try_join_all(futures).await?;

        for (key, stars) in batch_results {
            results.insert(key, stars);
        }

        Ok(results)
    }

    /// Health check - verify database connection
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }
}

/// Convert Cartesian direction to RA/Dec (degrees)
fn cartesian_to_ra_dec(v: glam::Vec3) -> (f64, f64) {
    let ra = (v.y.atan2(v.x)).to_degrees() as f64;
    let dec = (v.z / v.length()).asin().to_degrees() as f64;
    (ra, dec)
}

/// Angular size of tile at given LOD level (degrees)
fn angular_size_for_level(level: u8) -> f64 {
    // Level 0 = ~60° per tile (6 faces of cube)
    // Each level halves the angular size
    let base_angle = 60.0;
    base_angle / (1 << level) as f64
}

/// Magnitude limit for LOD level (brighter = lower magnitude)
fn magnitude_limit_for_level(level: u8) -> f64 {
    match level {
        0..=2 => 2.0,  // Brightest stars only
        3..=4 => 4.0,  // Naked eye visible
        5..=6 => 6.0,  // Binocular limit
        7..=8 => 8.0,  // Small telescope
        _ => 10.0,     // Deep survey
    }
}
