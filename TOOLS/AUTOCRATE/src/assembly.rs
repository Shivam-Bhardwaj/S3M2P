//! Assembly tree structure for crate components

use crate::{geometry::*, constants::LumberSize};
use serde::{Deserialize, Serialize};

/// Unique identifier for assembly components
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId(pub u32);

/// Rotation in Euler angles (radians)
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Rotation {
    pub rx: f32,  // Roll (around X axis)
    pub ry: f32,  // Pitch (around Y axis)
    pub rz: f32,  // Yaw (around Z axis)
}

impl Rotation {
    pub fn identity() -> Self {
        Self::default()
    }

    pub fn from_z(rz: f32) -> Self {
        Self { rx: 0.0, ry: 0.0, rz }
    }
}

/// Transform relative to parent coordinate system
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct LocalTransform {
    pub translation: Point3,
    pub rotation: Rotation,
}

impl LocalTransform {
    pub fn identity() -> Self {
        Self::default()
    }

    pub fn from_translation(t: Point3) -> Self {
        Self {
            translation: t,
            rotation: Rotation::identity()
        }
    }
}

/// Component types in the assembly
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ComponentType {
    // Assemblies
    CrateAssembly,
    BaseAssembly,
    WallAssembly(PanelType),

    // Parts
    Skid { lumber: LumberSize, length: f32 },
    Floorboard { lumber: LumberSize, length: f32 },
    Cleat { lumber: LumberSize, length: f32, is_vertical: bool },
    Panel { thickness: f32, width: f32, height: f32, panel_type: PanelType },

    // Fasteners
    Nail { x: f32, y: f32, z: f32 },
}

/// Assembly node in the tree
#[derive(Clone, Debug)]
pub struct AssemblyNode {
    pub id: ComponentId,
    pub name: String,
    pub component_type: ComponentType,
    pub transform: LocalTransform,
    pub bounds: BoundingBox,
    pub children: Vec<ComponentId>,
}

/// Complete assembly structure
#[derive(Clone, Debug)]
pub struct CrateAssembly {
    pub nodes: Vec<AssemblyNode>,
    pub root_id: ComponentId,
    next_id: u32,
}

impl CrateAssembly {
    pub fn new() -> Self {
        let root = AssemblyNode {
            id: ComponentId(0),
            name: "CrateAssembly".to_string(),
            component_type: ComponentType::CrateAssembly,
            transform: LocalTransform::identity(),
            bounds: BoundingBox::default(),
            children: Vec::new(),
        };

        Self {
            nodes: vec![root],
            root_id: ComponentId(0),
            next_id: 1,
        }
    }

    pub fn create_node(
        &mut self,
        name: String,
        component_type: ComponentType,
        transform: LocalTransform,
        bounds: BoundingBox,
    ) -> ComponentId {
        let id = ComponentId(self.next_id);
        self.next_id += 1;

        let node = AssemblyNode {
            id,
            name,
            component_type,
            transform,
            bounds,
            children: Vec::new(),
        };

        self.nodes.push(node);
        id
    }

    pub fn add_child(&mut self, parent: ComponentId, child: ComponentId) {
        if let Some(parent_node) = self.nodes.iter_mut().find(|n| n.id == parent) {
            parent_node.children.push(child);
        }
    }

    pub fn get_node(&self, id: ComponentId) -> Option<&AssemblyNode> {
        self.nodes.iter().find(|n| n.id == id)
    }
}

impl Default for CrateAssembly {
    fn default() -> Self {
        Self::new()
    }
}
