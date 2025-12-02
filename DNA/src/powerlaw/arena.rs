//! NetworkArena - SoA storage for network nodes
//!
//! Following the BoidArena pattern from DNA/src/lib.rs, provides:
//! - Fixed-capacity arrays for zero-allocation operation
//! - Generational indices for safe node references
//! - O(1) spawn/kill operations via free list
//! - Cache-friendly SoA layout

use glam::Vec2;

/// Generational index for safe node references
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeHandle {
    index: u16,
    generation: u16,
}

impl NodeHandle {
    pub const INVALID: Self = Self {
        index: u16::MAX,
        generation: 0,
    };

    #[inline]
    pub fn is_valid(&self) -> bool {
        self.index != u16::MAX
    }

    #[inline]
    pub fn index(&self) -> usize {
        self.index as usize
    }

    #[inline]
    pub fn generation(&self) -> u16 {
        self.generation
    }
}

/// Node properties
#[derive(Clone, Copy, Debug)]
pub struct NodeProperties {
    pub fitness: f32,     // Attractiveness for attachment (default 1.0)
    pub capacity: u16,    // Max outgoing connections
    pub resilience: f32,  // Resistance to cascade (default 1.0)
}

impl Default for NodeProperties {
    fn default() -> Self {
        Self {
            fitness: 1.0,
            resilience: 1.0,
            capacity: 50,
        }
    }
}

/// Network node storage using Structure of Arrays layout
///
/// Hot data (frequently accessed together) is grouped for cache locality.
/// Follows the pattern from DNA/src/lib.rs BoidArena (lines 257-408).
pub struct NetworkArena<const CAPACITY: usize> {
    // === HOT DATA (accessed every preferential attachment sample) ===
    /// Node degrees - number of connections
    pub degrees: Vec<u32>,
    /// Cumulative degree prefix sums for binary search sampling
    pub degree_prefix: Vec<u64>,
    /// Node active status
    pub alive: Vec<bool>,
    /// Generational counters for handle validation
    gen: Vec<u16>,

    // === COLD DATA (less frequently accessed) ===
    /// Node positions (for spatial layout/visualization)
    pub positions: Vec<Vec2>,
    /// Node weights/fitness for preferential attachment
    pub fitness: Vec<f32>,
    /// Node resources (for wealth distribution)
    pub resource: Vec<f32>,
    /// Node properties
    pub properties: Vec<NodeProperties>,

    // === MANAGEMENT ===
    /// Free list for O(1) spawn/kill
    free_list: Vec<u16>,
    free_count: usize,
    /// Number of alive nodes
    pub alive_count: usize,
    /// Sum of all degrees (for normalization)
    pub total_degree: u64,
    /// Flag indicating prefix sums need rebuild
    prefix_valid: bool,
}

impl<const CAPACITY: usize> NetworkArena<CAPACITY> {
    pub fn new() -> Self {
        let mut arena = Self {
            degrees: vec![0; CAPACITY],
            degree_prefix: vec![0; CAPACITY],
            alive: vec![false; CAPACITY],
            gen: vec![0; CAPACITY],
            positions: vec![Vec2::ZERO; CAPACITY],
            fitness: vec![1.0; CAPACITY],
            resource: vec![0.0; CAPACITY],
            properties: vec![NodeProperties::default(); CAPACITY],
            free_list: (0..CAPACITY as u16).collect(),
            free_count: CAPACITY,
            alive_count: 0,
            total_degree: 0,
            prefix_valid: false,
        };
        arena
    }

    /// Spawn a new node. O(1) operation.
    ///
    /// Returns a handle to the spawned node, or NodeHandle::INVALID if arena is full.
    pub fn spawn(&mut self, position: Vec2, props: NodeProperties) -> NodeHandle {
        if self.free_count == 0 {
            return NodeHandle::INVALID;
        }

        self.free_count -= 1;
        let idx = self.free_list[self.free_count] as usize;

        self.degrees[idx] = 0;
        self.alive[idx] = true;
        self.gen[idx] = self.gen[idx].wrapping_add(1);
        self.positions[idx] = position;
        self.fitness[idx] = props.fitness;
        self.resource[idx] = 0.0;
        self.properties[idx] = props;
        self.alive_count += 1;
        self.prefix_valid = false;

        NodeHandle {
            index: idx as u16,
            generation: self.gen[idx],
        }
    }

    /// Spawn a node with default properties
    #[inline]
    pub fn spawn_default(&mut self, position: Vec2) -> NodeHandle {
        self.spawn(position, NodeProperties::default())
    }

    /// Kill a node. O(1) operation.
    ///
    /// Does not immediately remove edges - caller must handle edge cleanup.
    pub fn kill(&mut self, handle: NodeHandle) -> bool {
        let idx = handle.index();
        if idx >= CAPACITY || !self.alive[idx] || self.gen[idx] != handle.generation() {
            return false;
        }

        self.alive[idx] = false;
        self.free_list[self.free_count] = handle.index as u16;
        self.free_count += 1;
        self.alive_count -= 1;
        self.total_degree = self.total_degree.saturating_sub(self.degrees[idx] as u64);
        self.prefix_valid = false;

        true
    }

    /// Validate a node handle
    #[inline]
    pub fn is_valid(&self, handle: NodeHandle) -> bool {
        let idx = handle.index();
        idx < CAPACITY && self.alive[idx] && self.gen[idx] == handle.generation()
    }

    /// Increment degree of a node (called when edge added)
    #[inline]
    pub fn increment_degree(&mut self, idx: usize) {
        if idx < CAPACITY && self.alive[idx] {
            self.degrees[idx] += 1;
            self.total_degree += 1;
            self.prefix_valid = false;
        }
    }

    /// Decrement degree of a node (called when edge removed)
    #[inline]
    pub fn decrement_degree(&mut self, idx: usize) {
        if idx < CAPACITY && self.alive[idx] && self.degrees[idx] > 0 {
            self.degrees[idx] -= 1;
            self.total_degree = self.total_degree.saturating_sub(1);
            self.prefix_valid = false;
        }
    }

    /// Rebuild prefix sums for preferential attachment sampling
    ///
    /// Called lazily when prefix_valid is false. O(n) operation.
    pub fn rebuild_prefix_sums(&mut self) {
        if self.prefix_valid {
            return;
        }

        let mut cumsum = 0u64;
        for i in 0..CAPACITY {
            if self.alive[i] {
                let weight = (self.degrees[i] as f32 * self.fitness[i]) as u64;
                cumsum += weight;
            }
            self.degree_prefix[i] = cumsum;
        }
        self.total_degree = cumsum;
        self.prefix_valid = true;
    }

    /// Sample node using preferential attachment (binary search)
    ///
    /// Returns None if no nodes alive. O(log n) operation.
    pub fn sample_preferential(&mut self, rng: &mut impl rand::Rng) -> Option<usize> {
        if self.alive_count == 0 || self.total_degree == 0 {
            return None;
        }

        // Rebuild prefix sums if dirty
        self.rebuild_prefix_sums();

        let target = rng.gen_range(0..self.total_degree);

        // Binary search for target in prefix sums
        let mut lo = 0;
        let mut hi = CAPACITY;
        while lo < hi {
            let mid = (lo + hi) / 2;
            if self.degree_prefix[mid] < target {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }

        // Find next alive node
        while lo < CAPACITY && !self.alive[lo] {
            lo += 1;
        }

        if lo < CAPACITY && self.alive[lo] {
            Some(lo)
        } else {
            None
        }
    }

    /// Iterate over alive node indices
    #[inline]
    pub fn iter_alive(&self) -> impl Iterator<Item = usize> + '_ {
        (0..CAPACITY).filter(|&i| self.alive[i])
    }

    /// Get capacity
    #[inline]
    pub fn capacity(&self) -> usize {
        CAPACITY
    }
}

impl<const CAPACITY: usize> Default for NetworkArena<CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_spawn_kill() {
        let mut arena: NetworkArena<100> = NetworkArena::new();
        assert_eq!(arena.alive_count, 0);

        let h1 = arena.spawn_default(Vec2::ZERO);
        assert!(h1.is_valid());
        assert_eq!(arena.alive_count, 1);

        let h2 = arena.spawn_default(Vec2::new(1.0, 1.0));
        assert!(h2.is_valid());
        assert_eq!(arena.alive_count, 2);

        assert!(arena.kill(h1));
        assert_eq!(arena.alive_count, 1);
        assert!(!arena.is_valid(h1));
        assert!(arena.is_valid(h2));
    }

    #[test]
    fn test_generational_index() {
        let mut arena: NetworkArena<10> = NetworkArena::new();

        let h1 = arena.spawn_default(Vec2::ZERO);
        let gen1 = h1.generation();

        arena.kill(h1);
        let h2 = arena.spawn_default(Vec2::ZERO);

        // Generation should have incremented
        assert_eq!(h2.index(), h1.index());
        assert_ne!(h2.generation(), gen1);
        assert!(!arena.is_valid(h1));
        assert!(arena.is_valid(h2));
    }

    #[test]
    fn test_preferential_attachment_bias() {
        let mut arena: NetworkArena<100> = NetworkArena::new();
        let mut rng = rand::thread_rng();

        let h1 = arena.spawn_default(Vec2::ZERO);
        let h2 = arena.spawn_default(Vec2::ZERO);

        // Give h1 much higher degree
        arena.degrees[h1.index()] = 10;
        arena.degrees[h2.index()] = 1;
        arena.total_degree = 11;

        // Sample many times
        let mut h1_count = 0;
        for _ in 0..1000 {
            if let Some(idx) = arena.sample_preferential(&mut rng) {
                if idx == h1.index() {
                    h1_count += 1;
                }
            }
        }

        // h1 should dominate (should be ~90% of samples)
        assert!(
            h1_count > 800,
            "Expected high-degree node to dominate, got {}/1000",
            h1_count
        );
    }
}
