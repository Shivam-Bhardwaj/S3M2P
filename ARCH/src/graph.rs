//! Dependency graph data structures

use serde::{Deserialize, Serialize};

/// Layer in the architecture hierarchy
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrateLayer {
    /// DNA - Core algorithms (lowest level)
    Dna,
    /// CORE - Domain-specific engines
    Core,
    /// PROJECT - Applications (WELCOME, HELIOS, etc.)
    Project,
    /// TOOL - Utilities (TOOLS/*, LEARN/*, SIMULATIONS/*)
    Tool,
}

/// Information about a single crate
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrateInfo {
    /// Crate name (from Cargo.toml)
    pub name: String,
    /// Path relative to workspace root
    pub path: String,
    /// Architecture layer
    pub layer: CrateLayer,
    /// Dependencies within workspace
    pub dependencies: Vec<String>,
}

/// Full dependency graph for the workspace
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// All crates in the workspace
    pub crates: Vec<CrateInfo>,
}

impl DependencyGraph {
    /// Create a new empty graph
    pub fn new() -> Self {
        Self { crates: Vec::new() }
    }

    /// Add a crate to the graph
    pub fn add_crate(&mut self, info: CrateInfo) {
        self.crates.push(info);
    }

    /// Get crate by name
    pub fn get(&self, name: &str) -> Option<&CrateInfo> {
        self.crates.iter().find(|c| c.name == name)
    }

    /// Get all crates that depend on the given crate
    pub fn dependents(&self, name: &str) -> Vec<&CrateInfo> {
        self.crates
            .iter()
            .filter(|c| c.dependencies.contains(&name.to_string()))
            .collect()
    }

    /// Get total dependency count
    pub fn edge_count(&self) -> usize {
        self.crates.iter().map(|c| c.dependencies.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_creation() {
        let mut graph = DependencyGraph::new();

        graph.add_crate(CrateInfo {
            name: "dna".to_string(),
            path: "DNA".to_string(),
            layer: CrateLayer::Dna,
            dependencies: vec![],
        });

        graph.add_crate(CrateInfo {
            name: "spice-engine".to_string(),
            path: "CORE/SPICE_ENGINE".to_string(),
            layer: CrateLayer::Core,
            dependencies: vec!["dna".to_string()],
        });

        assert_eq!(graph.crates.len(), 2);
        assert_eq!(graph.edge_count(), 1);
    }

    #[test]
    fn test_dependents() {
        let mut graph = DependencyGraph::new();

        graph.add_crate(CrateInfo {
            name: "dna".to_string(),
            path: "DNA".to_string(),
            layer: CrateLayer::Dna,
            dependencies: vec![],
        });

        graph.add_crate(CrateInfo {
            name: "engine-a".to_string(),
            path: "CORE/A".to_string(),
            layer: CrateLayer::Core,
            dependencies: vec!["dna".to_string()],
        });

        graph.add_crate(CrateInfo {
            name: "engine-b".to_string(),
            path: "CORE/B".to_string(),
            layer: CrateLayer::Core,
            dependencies: vec!["dna".to_string()],
        });

        let deps = graph.dependents("dna");
        assert_eq!(deps.len(), 2);
    }
}
