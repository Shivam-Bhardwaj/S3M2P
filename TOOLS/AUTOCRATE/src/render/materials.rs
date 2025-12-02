//! Material properties for rendering

use glam::Vec3;

/// Material type for different crate components
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaterialType {
    /// Softwood lumber (pine, fir, spruce)
    SoftwoodLumber,
    /// Hardwood lumber (oak, maple)
    HardwoodLumber,
    /// Plywood (CDX, BCX grades)
    Plywood,
    /// Galvanized steel (nails, screws)
    GalvanizedSteel,
    /// Bright steel (unfinished fasteners)
    BrightSteel,
}

/// Physical material properties for rendering
#[derive(Clone, Debug)]
pub struct Material {
    /// Material type
    pub material_type: MaterialType,
    /// Diffuse color (base color)
    pub diffuse: Vec3,
    /// Specular color (reflection color)
    pub specular: Vec3,
    /// Shininess exponent (Phong model)
    pub shininess: f32,
    /// Has texture flag
    pub has_texture: bool,
}

impl Material {
    /// Create material for softwood lumber
    pub fn softwood_lumber() -> Self {
        Self {
            material_type: MaterialType::SoftwoodLumber,
            diffuse: Vec3::new(0.85, 0.75, 0.60), // Light tan
            specular: Vec3::new(0.1, 0.1, 0.1),    // Low specular
            shininess: 8.0,                        // Matte finish
            has_texture: true,
        }
    }

    /// Create material for hardwood lumber
    pub fn hardwood_lumber() -> Self {
        Self {
            material_type: MaterialType::HardwoodLumber,
            diffuse: Vec3::new(0.65, 0.45, 0.30), // Darker brown
            specular: Vec3::new(0.2, 0.2, 0.2),    // Slight sheen
            shininess: 16.0,
            has_texture: true,
        }
    }

    /// Create material for plywood
    pub fn plywood() -> Self {
        Self {
            material_type: MaterialType::Plywood,
            diffuse: Vec3::new(0.80, 0.70, 0.55), // Light wood tone
            specular: Vec3::new(0.05, 0.05, 0.05), // Very matte
            shininess: 4.0,
            has_texture: true,
        }
    }

    /// Create material for galvanized steel (fasteners)
    pub fn galvanized_steel() -> Self {
        Self {
            material_type: MaterialType::GalvanizedSteel,
            diffuse: Vec3::new(0.60, 0.65, 0.70), // Dull silver-gray
            specular: Vec3::new(0.4, 0.4, 0.4),    // Moderate shine
            shininess: 32.0,
            has_texture: false,
        }
    }

    /// Create material for bright steel (unfinished)
    pub fn bright_steel() -> Self {
        Self {
            material_type: MaterialType::BrightSteel,
            diffuse: Vec3::new(0.75, 0.75, 0.75), // Light metallic gray
            specular: Vec3::new(0.8, 0.8, 0.8),    // High shine
            shininess: 64.0,
            has_texture: false,
        }
    }

    /// Get material by type
    pub fn from_type(material_type: MaterialType) -> Self {
        match material_type {
            MaterialType::SoftwoodLumber => Self::softwood_lumber(),
            MaterialType::HardwoodLumber => Self::hardwood_lumber(),
            MaterialType::Plywood => Self::plywood(),
            MaterialType::GalvanizedSteel => Self::galvanized_steel(),
            MaterialType::BrightSteel => Self::bright_steel(),
        }
    }
}

/// Lighting configuration
#[derive(Clone, Debug)]
pub struct Lighting {
    /// Directional light direction (normalized)
    pub light_dir: Vec3,
    /// Light color/intensity
    pub light_color: Vec3,
    /// Ambient light color
    pub ambient_color: Vec3,
}

impl Default for Lighting {
    fn default() -> Self {
        Self {
            // Light from upper-right-front
            light_dir: Vec3::new(0.5, 0.5, 0.7).normalize(),
            // Slightly warm white light
            light_color: Vec3::new(1.0, 0.98, 0.95),
            // Dark ambient (too.foo theme)
            ambient_color: Vec3::new(0.15, 0.15, 0.2),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_creation() {
        let lumber = Material::softwood_lumber();
        assert!(lumber.has_texture);
        assert_eq!(lumber.material_type, MaterialType::SoftwoodLumber);

        let steel = Material::galvanized_steel();
        assert!(!steel.has_texture);
        assert!(steel.shininess > lumber.shininess);
    }

    #[test]
    fn test_lighting_normalized() {
        let lighting = Lighting::default();
        let len = lighting.light_dir.length();
        assert!((len - 1.0).abs() < 0.001, "Light direction should be normalized");
    }
}
