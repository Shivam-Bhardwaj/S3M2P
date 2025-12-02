//! Cascade dynamics for network contagion
//!
//! Implements SIR (Susceptible-Infected-Recovered) and threshold cascade models.

use crate::powerlaw::{EdgeArena, NetworkArena};
use rand::Rng;

/// Cascade state for each node
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum CascadeState {
    Susceptible = 0,
    Infected = 1,
    Recovered = 2,
}

/// Cascade simulation arena with SoA layout
pub struct CascadeArena<const CAPACITY: usize> {
    /// Cascade state per node
    pub states: Vec<CascadeState>,
    /// Time since infection
    pub infection_time: Vec<f32>,
    /// Time until recovery
    pub recovery_time: Vec<f32>,
    /// Frontier buffers for BFS (double-buffered)
    frontier_a: Vec<u16>,
    frontier_b: Vec<u16>,
    frontier_len: usize,
    use_a: bool,
}

impl<const CAPACITY: usize> CascadeArena<CAPACITY> {
    pub fn new() -> Self {
        Self {
            states: vec![CascadeState::Susceptible; CAPACITY],
            infection_time: vec![0.0; CAPACITY],
            recovery_time: vec![1.0; CAPACITY],
            frontier_a: vec![0; CAPACITY],
            frontier_b: vec![0; CAPACITY],
            frontier_len: 0,
            use_a: true,
        }
    }

    /// Initialize cascade from seed nodes
    pub fn seed(&mut self, seeds: &[usize], recovery_rate: f32) {
        for &seed in seeds {
            if seed < CAPACITY {
                self.states[seed] = CascadeState::Infected;
                self.infection_time[seed] = 0.0;
                self.recovery_time[seed] = 1.0 / recovery_rate;
            }
        }

        // Initialize frontier
        self.frontier_len = seeds.len().min(CAPACITY);
        for (i, &seed) in seeds.iter().take(CAPACITY).enumerate() {
            self.frontier_a[i] = seed as u16;
        }
        self.use_a = true;
    }

    /// Single cascade propagation step (SIR model)
    ///
    /// Returns (new_infections, new_recoveries)
    pub fn step<const N: usize, const E: usize>(
        &mut self,
        network: &NetworkArena<N>,
        edges: &EdgeArena<N, E>,
        infection_prob: f32,
        dt: f32,
        rng: &mut impl Rng,
    ) -> (usize, usize) {
        let mut new_infections = 0;
        let mut new_recoveries = 0;

        let (frontier, next_frontier) = if self.use_a {
            (&self.frontier_a, &mut self.frontier_b)
        } else {
            (&self.frontier_b, &mut self.frontier_a)
        };

        let mut next_len = 0;

        // Process infected nodes
        for i in 0..self.frontier_len {
            let node_idx = frontier[i] as usize;
            if node_idx >= CAPACITY || !network.alive[node_idx] {
                continue;
            }

            // Update infection time
            self.infection_time[node_idx] += dt;

            // Check recovery
            if self.infection_time[node_idx] >= self.recovery_time[node_idx] {
                self.states[node_idx] = CascadeState::Recovered;
                new_recoveries += 1;
                continue;
            }

            // Spread to neighbors
            let neighbors = edges.neighbors(node_idx);
            for &neighbor in neighbors {
                let n = neighbor as usize;
                if n >= CAPACITY || !network.alive[n] {
                    continue;
                }

                if self.states[n] == CascadeState::Susceptible {
                    if rng.gen::<f32>() < infection_prob * dt {
                        self.states[n] = CascadeState::Infected;
                        self.infection_time[n] = 0.0;
                        new_infections += 1;

                        if next_len < CAPACITY {
                            next_frontier[next_len] = neighbor;
                            next_len += 1;
                        }
                    }
                }
            }

            // Keep in frontier if still infected
            if next_len < CAPACITY {
                next_frontier[next_len] = node_idx as u16;
                next_len += 1;
            }
        }

        self.frontier_len = next_len;
        self.use_a = !self.use_a;

        (new_infections, new_recoveries)
    }

    /// Get counts by state
    pub fn counts(&self) -> (usize, usize, usize) {
        let mut s = 0;
        let mut i = 0;
        let mut r = 0;

        for &state in &self.states {
            match state {
                CascadeState::Susceptible => s += 1,
                CascadeState::Infected => i += 1,
                CascadeState::Recovered => r += 1,
            }
        }

        (s, i, r)
    }

    /// Reset all to susceptible
    pub fn reset(&mut self) {
        self.states.fill(CascadeState::Susceptible);
        self.infection_time.fill(0.0);
        self.frontier_len = 0;
    }
}

impl<const CAPACITY: usize> Default for CascadeArena<CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}
