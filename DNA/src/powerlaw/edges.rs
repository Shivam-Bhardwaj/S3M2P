//! EdgeArena - Compressed edge storage for networks
//!
//! Uses a CSR-inspired (Compressed Sparse Row) layout for efficient edge storage.
//! Each node has a fixed capacity for outgoing edges to maintain zero-allocation.

/// Edge storage with fixed capacity per node
///
/// Uses CSR-like structure:
/// - row_ptr[i] = start index in edge_targets for node i's edges
/// - edge_targets[row_ptr[i]..row_ptr[i+1]] = neighbors of node i
pub struct EdgeArena<const NODE_CAP: usize, const EDGES_PER_NODE: usize> {
    /// Fixed adjacency lists - each node can have up to EDGES_PER_NODE neighbors
    pub targets: Vec<[u16; EDGES_PER_NODE]>,
    /// Edge weights (optional)
    pub weights: Vec<[f32; EDGES_PER_NODE]>,
    /// Number of edges per node
    pub edge_counts: Vec<u8>,
    /// Total edge count
    pub edge_count: usize,
}

impl<const NODE_CAP: usize, const EDGES_PER_NODE: usize> EdgeArena<NODE_CAP, EDGES_PER_NODE> {
    pub fn new() -> Self {
        Self {
            targets: vec![[0u16; EDGES_PER_NODE]; NODE_CAP],
            weights: vec![[1.0f32; EDGES_PER_NODE]; NODE_CAP],
            edge_counts: vec![0u8; NODE_CAP],
            edge_count: 0,
        }
    }

    /// Add an undirected edge between two nodes. O(1) operation.
    ///
    /// Returns true if edge was added, false if either node is at capacity.
    pub fn add_edge(&mut self, from: usize, to: usize, weight: f32) -> bool {
        if from >= NODE_CAP || to >= NODE_CAP {
            return false;
        }

        // Check if either node is at capacity
        if self.edge_counts[from] >= EDGES_PER_NODE as u8
            || self.edge_counts[to] >= EDGES_PER_NODE as u8
        {
            return false;
        }

        // Check if edge already exists
        if self.has_edge(from, to) {
            return false;
        }

        // Add bidirectional edge
        let from_count = self.edge_counts[from] as usize;
        let to_count = self.edge_counts[to] as usize;

        self.targets[from][from_count] = to as u16;
        self.weights[from][from_count] = weight;
        self.edge_counts[from] += 1;

        self.targets[to][to_count] = from as u16;
        self.weights[to][to_count] = weight;
        self.edge_counts[to] += 1;

        self.edge_count += 1;
        true
    }

    /// Check if edge exists from -> to. O(degree) operation.
    #[inline]
    pub fn has_edge(&self, from: usize, to: usize) -> bool {
        if from >= NODE_CAP {
            return false;
        }

        let count = self.edge_counts[from] as usize;
        for i in 0..count {
            if self.targets[from][i] == to as u16 {
                return true;
            }
        }
        false
    }

    /// Get neighbors of a node
    #[inline]
    pub fn neighbors(&self, node: usize) -> &[u16] {
        if node >= NODE_CAP {
            return &[];
        }
        let count = self.edge_counts[node] as usize;
        &self.targets[node][..count]
    }

    /// Get degree of a node
    #[inline]
    pub fn degree(&self, node: usize) -> usize {
        if node >= NODE_CAP {
            return 0;
        }
        self.edge_counts[node] as usize
    }

    /// Clear all edges (resets to empty graph)
    pub fn clear(&mut self) {
        self.edge_counts.fill(0);
        self.edge_count = 0;
    }
}

impl<const NODE_CAP: usize, const EDGES_PER_NODE: usize> Default
    for EdgeArena<NODE_CAP, EDGES_PER_NODE>
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_edge() {
        let mut edges: EdgeArena<10, 5> = EdgeArena::new();

        assert!(edges.add_edge(0, 1, 1.0));
        assert_eq!(edges.edge_count, 1);
        assert_eq!(edges.degree(0), 1);
        assert_eq!(edges.degree(1), 1);

        // Check bidirectional
        assert!(edges.has_edge(0, 1));
        assert!(edges.has_edge(1, 0));
    }

    #[test]
    fn test_duplicate_edge() {
        let mut edges: EdgeArena<10, 5> = EdgeArena::new();

        assert!(edges.add_edge(0, 1, 1.0));
        assert!(!edges.add_edge(0, 1, 1.0)); // Duplicate
        assert_eq!(edges.edge_count, 1);
    }

    #[test]
    fn test_capacity_limit() {
        let mut edges: EdgeArena<10, 3> = EdgeArena::new();

        // Add 3 edges from node 0
        assert!(edges.add_edge(0, 1, 1.0));
        assert!(edges.add_edge(0, 2, 1.0));
        assert!(edges.add_edge(0, 3, 1.0));

        // 4th edge should fail (at capacity)
        assert!(!edges.add_edge(0, 4, 1.0));
        assert_eq!(edges.degree(0), 3);
    }

    #[test]
    fn test_neighbors() {
        let mut edges: EdgeArena<10, 5> = EdgeArena::new();

        edges.add_edge(0, 1, 1.0);
        edges.add_edge(0, 2, 1.0);

        let neighbors = edges.neighbors(0);
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&1));
        assert!(neighbors.contains(&2));
    }
}
