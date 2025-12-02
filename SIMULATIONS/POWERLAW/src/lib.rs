//! Power law network visualization
//!
//! Interactive visualization of preferential attachment, cascades, and power law phenomena.

use dna::powerlaw::*;
use glam::Vec2;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

const CAPACITY: usize = 1000;
const EDGES_PER_NODE: usize = 50;

#[wasm_bindgen]
pub struct PowerLawSim {
    network: NetworkArena<CAPACITY>,
    edges: EdgeArena<CAPACITY, EDGES_PER_NODE>,
    cascade: CascadeArena<CAPACITY>,
    time: f32,
    paused: bool,
}

#[derive(Serialize, Deserialize)]
pub struct VisualSnapshot {
    pub nodes: Vec<NodeVisual>,
    pub edges: Vec<(usize, usize)>,
    pub metrics: MetricsSnapshot,
}

#[derive(Serialize, Deserialize)]
pub struct NodeVisual {
    pub x: f32,
    pub y: f32,
    pub degree: u32,
    pub state: u8, // 0=Susceptible, 1=Infected, 2=Recovered
    pub resource: f32,
}

#[derive(Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub node_count: usize,
    pub edge_count: usize,
    pub avg_degree: f32,
    pub max_degree: u32,
    pub alpha_estimate: f32,
    pub gini: f32,
}

#[wasm_bindgen]
impl PowerLawSim {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));

        let mut network = NetworkArena::new();
        let mut edges = EdgeArena::new();
        let mut rng = thread_rng();

        // Generate BA network
        let ba = BarabasiAlbert::new(3, 2, 800.0, 600.0);
        ba.generate(&mut network, &mut edges, 100, &mut rng);

        Self {
            network,
            edges,
            cascade: CascadeArena::new(),
            time: 0.0,
            paused: false,
        }
    }

    #[wasm_bindgen]
    pub fn step(&mut self, dt: f32) {
        if self.paused {
            return;
        }

        self.time += dt;

        // Add new node occasionally
        if self.time % 1.0 < dt && self.network.alive_count < CAPACITY {
            let mut rng = thread_rng();
            let pos = Vec2::new(
                rng.gen_range(0.0..800.0),
                rng.gen_range(0.0..600.0),
            );
            let new_handle = self.network.spawn_default(pos);

            if new_handle.is_valid() {
                let new_idx = new_handle.index();
                // Connect to 2 nodes via preferential attachment
                for _ in 0..2 {
                    if let Some(target) = self.network.sample_preferential(&mut rng) {
                        if self.edges.add_edge(new_idx, target, 1.0) {
                            self.network.increment_degree(new_idx);
                            self.network.increment_degree(target);
                        }
                    }
                }
            }
        }

        // Update resource (degree-based accumulation)
        let alive_indices: Vec<usize> = self.network.iter_alive().collect();
        for idx in alive_indices {
            let degree_factor = (self.network.degrees[idx] as f32).powf(1.1);
            self.network.resource[idx] += degree_factor * dt * 0.1;
        }
    }

    #[wasm_bindgen]
    pub fn get_snapshot(&self) -> JsValue {
        let mut nodes = Vec::new();
        let mut edges_list = Vec::new();

        for idx in self.network.iter_alive() {
            nodes.push(NodeVisual {
                x: self.network.positions[idx].x,
                y: self.network.positions[idx].y,
                degree: self.network.degrees[idx],
                state: self.cascade.states[idx] as u8,
                resource: self.network.resource[idx],
            });

            // Collect edges
            for &neighbor in self.edges.neighbors(idx) {
                let n = neighbor as usize;
                if n > idx && self.network.alive[n] {
                    edges_list.push((idx, n));
                }
            }
        }

        let metrics_obj = NetworkMetrics::compute(&self.network, &self.edges);

        let snapshot = VisualSnapshot {
            nodes,
            edges: edges_list,
            metrics: MetricsSnapshot {
                node_count: metrics_obj.node_count,
                edge_count: metrics_obj.edge_count,
                avg_degree: metrics_obj.avg_degree,
                max_degree: metrics_obj.max_degree,
                alpha_estimate: metrics_obj.alpha_estimate,
                gini: metrics_obj.gini_coefficient,
            },
        };

        serde_wasm_bindgen::to_value(&snapshot).unwrap()
    }

    #[wasm_bindgen]
    pub fn trigger_cascade(&mut self, node_idx: usize) {
        if node_idx < CAPACITY && self.network.alive[node_idx] {
            self.cascade.seed(&[node_idx], 0.1);
        }
    }

    #[wasm_bindgen]
    pub fn step_cascade(&mut self, dt: f32) {
        let mut rng = thread_rng();
        self.cascade.step(&self.network, &self.edges, 0.3, dt, &mut rng);
    }

    #[wasm_bindgen]
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) {
        let mut rng = thread_rng();
        self.network = NetworkArena::new();
        self.edges = EdgeArena::new();
        self.cascade = CascadeArena::new();
        self.time = 0.0;

        let ba = BarabasiAlbert::new(3, 2, 800.0, 600.0);
        ba.generate(&mut self.network, &mut self.edges, 100, &mut rng);
    }
}
