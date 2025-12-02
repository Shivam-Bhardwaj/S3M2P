//! Preferential attachment network generation
//!
//! Implements the Barabási-Albert model for generating scale-free networks.

use crate::powerlaw::{EdgeArena, NetworkArena, NodeProperties};
use glam::Vec2;
use rand::Rng;

/// Barabási-Albert preferential attachment model
///
/// Generates scale-free networks with degree distribution P(k) ~ k^(-3).
/// "Rich get richer" - new nodes preferentially connect to high-degree nodes.
pub struct BarabasiAlbert {
    /// Initial fully connected nodes
    pub m0: usize,
    /// Edges per new node (m <= m0)
    pub m: usize,
    /// Width of the spatial layout
    pub width: f32,
    /// Height of the spatial layout
    pub height: f32,
}

impl BarabasiAlbert {
    pub fn new(m0: usize, m: usize, width: f32, height: f32) -> Self {
        assert!(m <= m0, "m must be <= m0");
        assert!(m > 0, "m must be > 0");
        Self { m0, m, width, height }
    }

    /// Generate network into arena using preferential attachment
    ///
    /// Algorithm:
    /// 1. Create m0 initial nodes in a complete graph
    /// 2. For each new node:
    ///    - Connect to m existing nodes
    ///    - Selection probability proportional to degree
    pub fn generate<const N: usize, const E: usize>(
        &self,
        network: &mut NetworkArena<N>,
        edges: &mut EdgeArena<N, E>,
        total_nodes: usize,
        rng: &mut impl Rng,
    ) {
        // Step 1: Initialize with m0 fully connected nodes
        let mut handles = Vec::new();
        for _ in 0..self.m0.min(total_nodes).min(N) {
            let pos = Vec2::new(
                rng.gen_range(0.0..self.width),
                rng.gen_range(0.0..self.height),
            );
            let handle = network.spawn_default(pos);
            if handle.is_valid() {
                handles.push(handle.index());
            }
        }

        // Fully connect initial nodes
        for i in 0..handles.len() {
            for j in (i + 1)..handles.len() {
                if edges.add_edge(handles[i], handles[j], 1.0) {
                    network.increment_degree(handles[i]);
                    network.increment_degree(handles[j]);
                }
            }
        }

        // Step 2: Add remaining nodes with preferential attachment
        for _ in self.m0..total_nodes {
            let pos = Vec2::new(
                rng.gen_range(0.0..self.width),
                rng.gen_range(0.0..self.height),
            );
            let new_handle = network.spawn_default(pos);

            if !new_handle.is_valid() {
                break; // Arena full
            }

            let new_idx = new_handle.index();

            // Connect to m existing nodes via preferential attachment
            let mut connected = 0;
            let mut attempts = 0;
            let max_attempts = self.m * 20;

            while connected < self.m && attempts < max_attempts {
                attempts += 1;

                // Sample target using preferential attachment
                if let Some(target_idx) = network.sample_preferential(rng) {
                    // Avoid self-loops and duplicate edges
                    if target_idx != new_idx && !edges.has_edge(new_idx, target_idx) {
                        if edges.add_edge(new_idx, target_idx, 1.0) {
                            network.increment_degree(new_idx);
                            network.increment_degree(target_idx);
                            connected += 1;
                        }
                    }
                }
            }
        }
    }

    /// Generate network with specific node properties
    pub fn generate_with_properties<const N: usize, const E: usize, F, R>(
        &self,
        network: &mut NetworkArena<N>,
        edges: &mut EdgeArena<N, E>,
        total_nodes: usize,
        node_props: F,
        rng: &mut R,
    ) where
        F: Fn(&mut R) -> NodeProperties,
        R: Rng,
    {
        // Initial m0 nodes
        let mut handles = Vec::new();
        for _ in 0..self.m0.min(total_nodes).min(N) {
            let pos = Vec2::new(
                rng.gen_range(0.0..self.width),
                rng.gen_range(0.0..self.height),
            );
            let props = node_props(rng);
            let handle = network.spawn(pos, props);
            if handle.is_valid() {
                handles.push(handle.index());
            }
        }

        // Fully connect initial nodes
        for i in 0..handles.len() {
            for j in (i + 1)..handles.len() {
                if edges.add_edge(handles[i], handles[j], 1.0) {
                    network.increment_degree(handles[i]);
                    network.increment_degree(handles[j]);
                }
            }
        }

        // Add remaining nodes
        for _ in self.m0..total_nodes {
            let pos = Vec2::new(
                rng.gen_range(0.0..self.width),
                rng.gen_range(0.0..self.height),
            );
            let props = node_props(rng);
            let new_handle = network.spawn(pos, props);

            if !new_handle.is_valid() {
                break;
            }

            let new_idx = new_handle.index();
            let mut connected = 0;
            let mut attempts = 0;

            while connected < self.m && attempts < self.m * 20 {
                attempts += 1;
                if let Some(target_idx) = network.sample_preferential(rng) {
                    if target_idx != new_idx && !edges.has_edge(new_idx, target_idx) {
                        if edges.add_edge(new_idx, target_idx, 1.0) {
                            network.increment_degree(new_idx);
                            network.increment_degree(target_idx);
                            connected += 1;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ba_network_generation() {
        let mut network: NetworkArena<100> = NetworkArena::new();
        let mut edges: EdgeArena<100, 50> = EdgeArena::new();
        let mut rng = rand::thread_rng();

        let ba = BarabasiAlbert::new(3, 2, 100.0, 100.0);
        ba.generate(&mut network, &mut edges, 50, &mut rng);

        // Should have created 50 nodes
        assert_eq!(network.alive_count, 50);

        // Should have created edges (exact count depends on capacity)
        assert!(edges.edge_count > 0);

        // Initial m0 nodes should be fully connected
        // m0=3 -> 3 edges among initial nodes
        assert!(edges.edge_count >= 3);
    }

    #[test]
    fn test_ba_scale_free_property() {
        let mut network: NetworkArena<200> = NetworkArena::new();
        let mut edges: EdgeArena<200, 100> = EdgeArena::new();
        let mut rng = rand::thread_rng();

        let ba = BarabasiAlbert::new(3, 2, 200.0, 200.0);
        ba.generate(&mut network, &mut edges, 100, &mut rng);

        // Collect degree distribution
        let mut degrees = Vec::new();
        for i in 0..200 {
            if network.alive[i] {
                degrees.push(network.degrees[i]);
            }
        }

        // In BA model, should have some high-degree hubs
        let max_degree = degrees.iter().max().copied().unwrap_or(0);
        let avg_degree = degrees.iter().sum::<u32>() as f32 / degrees.len() as f32;

        // Max degree should be significantly higher than average (characteristic of scale-free)
        assert!(max_degree as f32 > avg_degree * 2.0,
            "Expected scale-free hub formation, max={}, avg={}",
            max_degree, avg_degree);

        // Most nodes should have low degree (power law)
        let low_degree_count = degrees.iter().filter(|&&d| d <= 4).count();
        assert!(low_degree_count as f32 > degrees.len() as f32 * 0.5,
            "Expected many low-degree nodes in power law distribution");
    }
}
