//! Rendering module for AutoCrate visualization
//!
//! Provides both 3D (WebGL2) and 2D (Canvas2D) rendering:
//! - `webgl` - 3D interactive visualization with orbit controls
//! - `canvas2d` - 2D technical drawing with dimensions
//! - `textures` - Procedural wood grain generation
//! - `materials` - Visual material properties (lumber, plywood, metal)

pub mod webgl;
pub mod canvas2d;
pub mod textures;
pub mod materials;
pub mod mesh;

pub use webgl::{WebGLRenderer, Camera, RenderMode};
pub use canvas2d::Canvas2DRenderer;
pub use textures::{WoodTexture, generate_wood_grain};
pub use materials::{Material, MaterialType};
pub use mesh::{Mesh, MeshBuffer};

/// View mode toggle
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewMode {
    /// 3D interactive view with orbit controls
    ThreeD,
    /// 2D technical drawing (orthographic projection)
    TwoD,
}
