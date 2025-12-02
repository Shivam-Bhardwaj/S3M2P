//! Procedural wood texture generation

use rand::Rng;
use web_sys::ImageData;

/// Wood species for texture generation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WoodSpecies {
    Pine,
    Fir,
    Spruce,
    Oak,
    Maple,
}

/// Wood texture parameters
#[derive(Clone, Debug)]
pub struct WoodTexture {
    pub species: WoodSpecies,
    pub width: u32,
    pub height: u32,
    pub seed: u32,
    pub grain_scale: f32,
    pub ring_spacing: f32,
    pub color_variation: f32,
}

impl WoodTexture {
    pub fn new(species: WoodSpecies, width: u32, height: u32) -> Self {
        Self {
            species,
            width,
            height,
            seed: 42,
            grain_scale: 1.0,
            ring_spacing: match species {
                WoodSpecies::Pine | WoodSpecies::Fir => 8.0,
                WoodSpecies::Spruce => 6.0,
                WoodSpecies::Oak | WoodSpecies::Maple => 12.0,
            },
            color_variation: 0.15,
        }
    }

    /// Get base color for species
    fn base_color(&self) -> (u8, u8, u8) {
        match self.species {
            WoodSpecies::Pine => (220, 190, 150),      // Light tan
            WoodSpecies::Fir => (210, 180, 140),       // Slightly darker tan
            WoodSpecies::Spruce => (230, 200, 160),    // Pale yellow
            WoodSpecies::Oak => (180, 140, 100),       // Brown
            WoodSpecies::Maple => (200, 170, 130),     // Light brown
        }
    }

    /// Get ring color (darker) for species
    fn ring_color(&self) -> (u8, u8, u8) {
        match self.species {
            WoodSpecies::Pine => (160, 130, 90),
            WoodSpecies::Fir => (150, 120, 80),
            WoodSpecies::Spruce => (170, 140, 100),
            WoodSpecies::Oak => (120, 80, 40),
            WoodSpecies::Maple => (140, 110, 70),
        }
    }
}

/// Generate procedural wood grain texture
pub fn generate_wood_grain(texture: &WoodTexture) -> Vec<u8> {
    let width = texture.width as usize;
    let height = texture.height as usize;
    let mut data = vec![0u8; width * height * 4]; // RGBA

    let (base_r, base_g, base_b) = texture.base_color();
    let (ring_r, ring_g, ring_b) = texture.ring_color();

    // Simple Perlin-like noise using sine waves
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;

            // Normalized coordinates
            let nx = x as f32 / width as f32;
            let ny = y as f32 / height as f32;

            // Distance from center (for growth rings)
            let cx = 0.5;
            let cy = 0.5;
            let dx = (nx - cx) * 2.0;
            let dy = (ny - cy) * 2.0;
            let dist = (dx * dx + dy * dy).sqrt();

            // Growth ring pattern (concentric circles with noise)
            let ring_freq = texture.ring_spacing;
            let ring_noise = (nx * 37.0 + ny * 23.0 + texture.seed as f32).sin() * 0.3;
            let ring_pattern = ((dist * ring_freq + ring_noise) * std::f32::consts::PI).sin();

            // Grain direction (vertical stripes with slight wave)
            let grain_noise = (nx * 100.0 * texture.grain_scale).sin() * 0.1 +
                              (ny * 50.0).sin() * 0.05;
            let grain_pattern = ((ny * 200.0 + grain_noise) * std::f32::consts::PI).sin();

            // Combine patterns
            let ring_weight = (ring_pattern * 0.5 + 0.5) * 0.7;
            let grain_weight = (grain_pattern * 0.5 + 0.5) * 0.3;
            let pattern = ring_weight + grain_weight;

            // Interpolate between base and ring color
            let r = lerp(base_r as f32, ring_r as f32, pattern);
            let g = lerp(base_g as f32, ring_g as f32, pattern);
            let b = lerp(base_b as f32, ring_b as f32, pattern);

            // Add slight color variation for realism
            let variation = (nx * 17.0 + ny * 13.0).sin() * texture.color_variation;
            let r = (r * (1.0 + variation)).clamp(0.0, 255.0) as u8;
            let g = (g * (1.0 + variation)).clamp(0.0, 255.0) as u8;
            let b = (b * (1.0 + variation)).clamp(0.0, 255.0) as u8;

            data[idx] = r;
            data[idx + 1] = g;
            data[idx + 2] = b;
            data[idx + 3] = 255; // Alpha
        }
    }

    data
}

/// Generate plywood edge texture (layered appearance)
pub fn generate_plywood_edge(width: u32, height: u32) -> Vec<u8> {
    let mut data = vec![0u8; (width * height * 4) as usize];
    let width_usize = width as usize;

    // Plywood has alternating light/dark horizontal layers
    for y in 0..height {
        // Layer thickness varies
        let layer_index = (y as f32 / 8.0).floor() as u32;
        let is_dark = layer_index % 2 == 1;

        let (r, g, b) = if is_dark {
            (140, 110, 80)  // Dark layer
        } else {
            (200, 170, 130) // Light layer
        };

        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            // Add slight horizontal grain
            let grain = ((x as f32 * 0.3).sin() * 5.0) as i16;

            data[idx] = (r as i16 + grain).clamp(0, 255) as u8;
            data[idx + 1] = (g as i16 + grain).clamp(0, 255) as u8;
            data[idx + 2] = (b as i16 + grain).clamp(0, 255) as u8;
            data[idx + 3] = 255;
        }
    }

    data
}

/// Create ImageData from raw RGBA bytes (for WebGL texture upload)
pub fn create_image_data(data: Vec<u8>, width: u32, height: u32) -> Result<ImageData, String> {
    use wasm_bindgen::Clamped;

    ImageData::new_with_u8_clamped_array(Clamped(&data), width)
        .map_err(|e| format!("Failed to create ImageData: {:?}", e))
}

/// Linear interpolation helper
#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wood_texture_creation() {
        let texture = WoodTexture::new(WoodSpecies::Pine, 256, 256);
        assert_eq!(texture.width, 256);
        assert_eq!(texture.height, 256);
        assert_eq!(texture.species, WoodSpecies::Pine);
    }

    #[test]
    fn test_generate_wood_grain() {
        let texture = WoodTexture::new(WoodSpecies::Oak, 64, 64);
        let data = generate_wood_grain(&texture);

        // Should have RGBA data
        assert_eq!(data.len(), 64 * 64 * 4);

        // Alpha should be 255
        for i in 0..64 * 64 {
            assert_eq!(data[i * 4 + 3], 255);
        }
    }

    #[test]
    fn test_plywood_edge() {
        let data = generate_plywood_edge(32, 32);
        assert_eq!(data.len(), 32 * 32 * 4);
    }

    #[test]
    fn test_lerp() {
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
        assert!((lerp(0.0, 10.0, 0.5) - 5.0).abs() < 0.001);
    }
}
