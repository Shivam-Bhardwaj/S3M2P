//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: graph_slam.rs | LEARN/learn_core/src/demos/graph_slam.rs
//! PURPOSE: Graph SLAM demo - pose graph optimization
//! MODIFIED: 2025-12-12
//! LAYER: LEARN → learn_core → demos
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! # Graph SLAM
//!
//! Represents SLAM as a graph optimization problem:
//! - Nodes: Robot poses at different times
//! - Edges: Constraints between poses (odometry, loop closures)
//!
//! Key insight: Accumulating drift can be corrected by adding loop closure
//! constraints and optimizing the entire graph to satisfy all constraints.

use crate::{Demo, ParamMeta, Rng, Vec2};

/// A node in the pose graph (robot pose at a specific time)
#[derive(Clone, Debug)]
pub struct PoseNode {
    /// Position (x, y)
    pub pos: Vec2,
    /// Heading angle
    pub theta: f32,
    /// Whether this is a keyframe (for visualization)
    pub is_keyframe: bool,
}

/// An edge constraint between two nodes
#[derive(Clone, Debug)]
pub struct GraphEdge {
    /// Index of source node
    pub from: usize,
    /// Index of target node
    pub to: usize,
    /// Measured displacement (dx, dy)
    pub delta: Vec2,
    /// Information weight (inverse variance)
    pub weight: f32,
    /// Whether this is a loop closure (vs odometry)
    pub is_loop_closure: bool,
}

/// Graph SLAM Demo
#[derive(Clone)]
pub struct GraphSlamDemo {
    // True robot state (for simulation)
    pub true_pos: Vec2,
    pub true_theta: f32,

    // The pose graph
    pub nodes: Vec<PoseNode>,
    pub edges: Vec<GraphEdge>,

    // True path (for comparison)
    pub true_path: Vec<Vec2>,

    // Parameters
    pub odometry_noise: f32,
    pub loop_threshold: f32,

    // Simulation state
    time: f32,
    rng: Rng,
    frame_count: u32,
    keyframe_interval: u32,

    // Optimization state
    pub is_optimizing: bool,
    pub optimization_iterations: u32,
    pub last_loop_closure: Option<(usize, usize)>,
}

impl Default for GraphSlamDemo {
    fn default() -> Self {
        Self {
            true_pos: Vec2::new(0.5, 0.3),
            true_theta: 0.0,
            nodes: Vec::new(),
            edges: Vec::new(),
            true_path: Vec::new(),
            odometry_noise: 0.005,    // BEST: minimum noise
            loop_threshold: 0.12,     // Distance to detect loop closure
            time: 0.0,
            rng: Rng::new(42),
            frame_count: 0,
            keyframe_interval: 15,    // Add node every N frames
            is_optimizing: false,
            optimization_iterations: 0,
            last_loop_closure: None,
        }
    }
}

impl GraphSlamDemo {
    fn gaussian(&mut self, std_dev: f32) -> f32 {
        let u1 = self.rng.range(0.0001, 1.0);
        let u2 = self.rng.range(0.0, 1.0);
        let z = (-2.0 * u1.ln()).sqrt() * (std::f32::consts::TAU * u2).cos();
        std_dev * z
    }

    /// Move the robot and add nodes/edges
    fn move_robot(&mut self, dt: f32) {
        self.time += dt;
        self.frame_count += 1;

        // Figure-8 motion for interesting loop closures
        let angular_vel = 0.4;
        let linear_vel = 0.06;

        // True motion
        self.true_theta += angular_vel * dt;
        let dx = linear_vel * dt * self.true_theta.cos();
        let dy = linear_vel * dt * self.true_theta.sin();

        self.true_pos.x = (self.true_pos.x + dx).clamp(0.1, 0.9);
        self.true_pos.y = (self.true_pos.y + dy).clamp(0.1, 0.9);

        // Record true path
        self.true_path.push(self.true_pos);
        if self.true_path.len() > 500 {
            self.true_path.remove(0);
        }

        // Add keyframe nodes periodically
        if self.frame_count % self.keyframe_interval == 0 {
            self.add_node(dt);
        }
    }

    fn add_node(&mut self, _dt: f32) {
        let node_idx = self.nodes.len();

        // Pre-generate noise values to avoid borrow conflicts
        let noise_x = self.gaussian(self.odometry_noise);
        let noise_y = self.gaussian(self.odometry_noise);
        let theta_noise = self.gaussian(0.1);

        // Calculate noisy pose based on accumulated odometry
        let noisy_pos = if node_idx == 0 {
            // First node: start at true position (with small noise)
            Vec2::new(
                self.true_pos.x + noise_x,
                self.true_pos.y + noise_y,
            )
        } else {
            // Subsequent nodes: add from previous with drift
            let prev_pos = self.nodes[node_idx - 1].pos;
            let true_path_idx = self.true_path.len().saturating_sub(self.keyframe_interval as usize + 1);
            let old_true_pos = self.true_path[true_path_idx];
            let true_dx = self.true_pos.x - old_true_pos.x;
            let true_dy = self.true_pos.y - old_true_pos.y;

            Vec2::new(
                prev_pos.x + true_dx + noise_x,
                prev_pos.y + true_dy + noise_y,
            )
        };

        // Add the node
        let new_node = PoseNode {
            pos: Vec2::new(
                noisy_pos.x.clamp(0.05, 0.95),
                noisy_pos.y.clamp(0.05, 0.95),
            ),
            theta: self.true_theta + theta_noise,
            is_keyframe: true,
        };
        self.nodes.push(new_node);

        // Add odometry edge to previous node
        if node_idx > 0 {
            let prev_pos = self.nodes[node_idx - 1].pos;
            let curr_pos = self.nodes[node_idx].pos;
            let odometry_noise = self.odometry_noise;

            self.edges.push(GraphEdge {
                from: node_idx - 1,
                to: node_idx,
                delta: Vec2::new(curr_pos.x - prev_pos.x, curr_pos.y - prev_pos.y),
                weight: 1.0 / (odometry_noise * odometry_noise),
                is_loop_closure: false,
            });
        }

        // Check for loop closures (to nodes not adjacent)
        self.detect_loop_closures(node_idx);
    }

    fn detect_loop_closures(&mut self, current_idx: usize) {
        if current_idx < 10 {
            return; // Need some history first
        }

        // Clone data needed for comparison
        let current_pos = self.nodes[current_idx].pos;
        let loop_threshold = self.loop_threshold;
        let nodes_len = self.nodes.len();
        let true_path_len = self.true_path.len();
        let odometry_noise = self.odometry_noise;

        // Pre-generate noise
        let noise_x = self.gaussian(odometry_noise * 0.5);
        let noise_y = self.gaussian(odometry_noise * 0.5);

        // Check against older nodes (not recent ones)
        for i in 0..(current_idx.saturating_sub(8)) {
            let other_pos = self.nodes[i].pos;

            let dx = current_pos.x - other_pos.x;
            let dy = current_pos.y - other_pos.y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < loop_threshold {
                // Found a loop closure!
                // Calculate what the constraint should be based on true positions
                let true_current_idx = true_path_len - 1;
                let true_other_idx = (i as f32 / nodes_len as f32 * true_path_len as f32) as usize;

                if true_other_idx < true_path_len && true_current_idx < true_path_len {
                    let true_current = self.true_path[true_current_idx];
                    let true_other = self.true_path[true_other_idx.min(true_path_len - 1)];

                    self.edges.push(GraphEdge {
                        from: i,
                        to: current_idx,
                        delta: Vec2::new(
                            true_current.x - true_other.x + noise_x,
                            true_current.y - true_other.y + noise_y,
                        ),
                        weight: 2.0 / (odometry_noise * odometry_noise), // Higher weight for loop closures
                        is_loop_closure: true,
                    });

                    self.last_loop_closure = Some((i, current_idx));
                }
                break; // One loop closure per node
            }
        }
    }

    /// Run one iteration of graph optimization (Gauss-Newton style)
    pub fn optimize_step(&mut self) {
        if self.nodes.len() < 2 {
            return;
        }

        self.optimization_iterations += 1;

        // Simple gradient descent optimization
        let learning_rate = 0.1;

        // For each edge, adjust nodes to satisfy constraint
        for edge in &self.edges {
            let from_pos = self.nodes[edge.from].pos;
            let to_pos = self.nodes[edge.to].pos;

            // Current displacement
            let current_delta = Vec2::new(to_pos.x - from_pos.x, to_pos.y - from_pos.y);

            // Error (difference from measured)
            let error = Vec2::new(
                current_delta.x - edge.delta.x,
                current_delta.y - edge.delta.y,
            );

            // Weighted gradient update
            let update = learning_rate * edge.weight;

            // Don't move first node (anchor)
            if edge.from > 0 {
                self.nodes[edge.from].pos.x += error.x * update * 0.5;
                self.nodes[edge.from].pos.y += error.y * update * 0.5;
            }

            self.nodes[edge.to].pos.x -= error.x * update * 0.5;
            self.nodes[edge.to].pos.y -= error.y * update * 0.5;
        }

        // Clamp positions
        for node in &mut self.nodes {
            node.pos.x = node.pos.x.clamp(0.05, 0.95);
            node.pos.y = node.pos.y.clamp(0.05, 0.95);
        }
    }

    /// Toggle optimization
    pub fn toggle_optimization(&mut self) {
        self.is_optimizing = !self.is_optimizing;
        if self.is_optimizing {
            self.optimization_iterations = 0;
        }
    }

    /// Manually trigger loop closure detection at current position
    pub fn add_manual_loop_closure(&mut self) {
        if self.nodes.len() < 2 {
            return;
        }

        let current_idx = self.nodes.len() - 1;
        let current = &self.nodes[current_idx];

        // Find closest older node
        let mut best_idx = None;
        let mut best_dist = f32::MAX;

        for i in 0..(current_idx.saturating_sub(5)) {
            let other = &self.nodes[i];
            let dx = current.pos.x - other.pos.x;
            let dy = current.pos.y - other.pos.y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < best_dist {
                best_dist = dist;
                best_idx = Some(i);
            }
        }

        if let Some(i) = best_idx {
            if best_dist < 0.3 { // More lenient for manual
                let true_current_idx = self.true_path.len() - 1;
                let true_other_idx = (i as f32 / self.nodes.len() as f32 * self.true_path.len() as f32) as usize;

                if true_other_idx < self.true_path.len() {
                    let true_current = self.true_path[true_current_idx];
                    let true_other = self.true_path[true_other_idx.min(self.true_path.len() - 1)];

                    self.edges.push(GraphEdge {
                        from: i,
                        to: current_idx,
                        delta: Vec2::new(
                            true_current.x - true_other.x,
                            true_current.y - true_other.y,
                        ),
                        weight: 3.0 / (self.odometry_noise * self.odometry_noise),
                        is_loop_closure: true,
                    });

                    self.last_loop_closure = Some((i, current_idx));
                }
            }
        }
    }

    /// Get total graph error (sum of squared constraint violations)
    pub fn graph_error(&self) -> f32 {
        let mut total = 0.0;

        for edge in &self.edges {
            let from_pos = self.nodes[edge.from].pos;
            let to_pos = self.nodes[edge.to].pos;

            let current_delta = Vec2::new(to_pos.x - from_pos.x, to_pos.y - from_pos.y);
            let error_x = current_delta.x - edge.delta.x;
            let error_y = current_delta.y - edge.delta.y;

            total += (error_x * error_x + error_y * error_y) * edge.weight;
        }

        total
    }

    /// Get drift error (distance from estimated to true final position)
    pub fn drift_error(&self) -> f32 {
        if self.nodes.is_empty() || self.true_path.is_empty() {
            return 0.0;
        }

        let est = &self.nodes[self.nodes.len() - 1];
        let true_pos = self.true_path[self.true_path.len() - 1];

        let dx = est.pos.x - true_pos.x;
        let dy = est.pos.y - true_pos.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Count loop closure edges
    pub fn loop_closure_count(&self) -> usize {
        self.edges.iter().filter(|e| e.is_loop_closure).count()
    }
}

impl Demo for GraphSlamDemo {
    fn reset(&mut self, seed: u64) {
        self.rng = Rng::new(seed);
        self.true_pos = Vec2::new(0.5, 0.3);
        self.true_theta = 0.0;
        self.nodes.clear();
        self.edges.clear();
        self.true_path.clear();
        self.time = 0.0;
        self.frame_count = 0;
        self.is_optimizing = false;
        self.optimization_iterations = 0;
        self.last_loop_closure = None;
    }

    fn step(&mut self, dt: f32) {
        self.move_robot(dt);

        // Run optimization if enabled
        if self.is_optimizing {
            self.optimize_step();
        }
    }

    fn set_param(&mut self, name: &str, value: f32) -> bool {
        match name {
            "odometry_noise" => {
                self.odometry_noise = value.clamp(0.005, 0.1);
                true
            }
            "loop_threshold" => {
                self.loop_threshold = value.clamp(0.05, 0.3);
                true
            }
            _ => false,
        }
    }

    fn params() -> &'static [ParamMeta] {
        &[
            ParamMeta {
                name: "odometry_noise",
                label: "Odometry Noise",
                min: 0.005,
                max: 0.1,
                step: 0.005,
                default: 0.005,   // BEST: minimum noise
            },
            ParamMeta {
                name: "loop_threshold",
                label: "Loop Threshold",
                min: 0.05,
                max: 0.3,
                step: 0.01,
                default: 0.12,    // Reasonable detection distance
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reset() {
        let mut demo = GraphSlamDemo::default();
        demo.step(0.1);
        demo.step(0.1);
        demo.reset(123);
        assert!(demo.nodes.is_empty());
        assert!(demo.edges.is_empty());
    }

    #[test]
    fn test_node_creation() {
        let mut demo = GraphSlamDemo::default();
        demo.keyframe_interval = 1; // Add node every frame

        for _ in 0..10 {
            demo.step(0.016);
        }

        assert!(!demo.nodes.is_empty());
        assert!(!demo.edges.is_empty());
    }

    #[test]
    fn test_optimization() {
        let mut demo = GraphSlamDemo::default();
        demo.keyframe_interval = 1;

        for _ in 0..20 {
            demo.step(0.016);
        }

        let error_before = demo.graph_error();

        for _ in 0..10 {
            demo.optimize_step();
        }

        let error_after = demo.graph_error();

        // Optimization should reduce error (or keep it similar if already good)
        assert!(error_after <= error_before + 0.1);
    }

    #[test]
    fn test_deterministic() {
        let mut demo1 = GraphSlamDemo::default();
        let mut demo2 = GraphSlamDemo::default();

        demo1.reset(42);
        demo2.reset(42);

        for _ in 0..50 {
            demo1.step(0.016);
            demo2.step(0.016);
        }

        assert!((demo1.true_pos.x - demo2.true_pos.x).abs() < 1e-6);
    }
}
