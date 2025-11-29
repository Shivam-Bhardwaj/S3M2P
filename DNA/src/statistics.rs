//! Population statistics and metrics
//!
//! Provides utilities for computing and tracking simulation metrics:
//! - Population counts by role
//! - Diversity indices
//! - Generation tracking
//! - Energy statistics
//!
//! ## Traceability
//! - Used by: too.foo (status display), future simulations
//! - Tests: test_population_metrics, test_diversity_score

use crate::{BoidArena, BoidRole};

/// Comprehensive population metrics snapshot
#[derive(Clone, Debug, Default)]
pub struct PopulationMetrics {
    pub total_alive: usize,
    pub herbivore_count: usize,
    pub carnivore_count: usize,
    pub scavenger_count: usize,
    pub max_generation: u16,
    pub avg_generation: f32,
    pub max_speed: f32,
    pub avg_speed: f32,
    pub avg_energy: f32,
    pub min_energy: f32,
    pub max_energy: f32,
    pub diversity: f32,
}

impl PopulationMetrics {
    /// Compute all metrics from a boid arena
    pub fn compute<const CAP: usize>(arena: &BoidArena<CAP>) -> Self {
        if arena.alive_count == 0 {
            return Self::default();
        }

        let mut metrics = Self {
            total_alive: arena.alive_count,
            min_energy: f32::MAX,
            ..Default::default()
        };

        let mut speed_sum = 0.0f32;
        let mut energy_sum = 0.0f32;
        let mut gen_sum = 0u64;

        for idx in arena.iter_alive() {
            // Role counts
            match arena.roles[idx] {
                BoidRole::Herbivore => metrics.herbivore_count += 1,
                BoidRole::Carnivore => metrics.carnivore_count += 1,
                BoidRole::Scavenger => metrics.scavenger_count += 1,
            }

            // Generation tracking
            let gen = arena.generation[idx];
            gen_sum += gen as u64;
            if gen > metrics.max_generation {
                metrics.max_generation = gen;
            }

            // Speed tracking
            let speed = arena.genes[idx].max_speed;
            speed_sum += speed;
            if speed > metrics.max_speed {
                metrics.max_speed = speed;
            }

            // Energy tracking
            let energy = arena.energy[idx];
            energy_sum += energy;
            if energy < metrics.min_energy {
                metrics.min_energy = energy;
            }
            if energy > metrics.max_energy {
                metrics.max_energy = energy;
            }
        }

        let n = metrics.total_alive as f32;
        metrics.avg_generation = gen_sum as f32 / n;
        metrics.avg_speed = speed_sum / n;
        metrics.avg_energy = energy_sum / n;

        // Compute diversity
        metrics.diversity = compute_diversity_score(
            metrics.herbivore_count,
            metrics.carnivore_count,
            metrics.scavenger_count,
            metrics.total_alive,
        );

        metrics
    }

    /// Get the dominant role in the population
    pub fn dominant_role(&self) -> Option<BoidRole> {
        if self.total_alive == 0 {
            return None;
        }

        let max = self
            .herbivore_count
            .max(self.carnivore_count)
            .max(self.scavenger_count);

        if max == self.herbivore_count {
            Some(BoidRole::Herbivore)
        } else if max == self.carnivore_count {
            Some(BoidRole::Carnivore)
        } else {
            Some(BoidRole::Scavenger)
        }
    }

    /// Get role distribution as fractions
    pub fn role_distribution(&self) -> (f32, f32, f32) {
        if self.total_alive == 0 {
            return (0.0, 0.0, 0.0);
        }
        let n = self.total_alive as f32;
        (
            self.herbivore_count as f32 / n,
            self.carnivore_count as f32 / n,
            self.scavenger_count as f32 / n,
        )
    }

    /// Check if population is healthy (diverse with balanced roles)
    pub fn is_healthy(&self) -> bool {
        self.diversity > 0.5 && self.total_alive >= 10
    }
}

/// Compute Shannon entropy-based diversity score from role counts
fn compute_diversity_score(
    herbivore_count: usize,
    carnivore_count: usize,
    scavenger_count: usize,
    total: usize,
) -> f32 {
    if total < 10 {
        return 1.0; // Too few to measure, assume diverse
    }

    let n = total as f32;
    let h_frac = herbivore_count as f32 / n;
    let c_frac = carnivore_count as f32 / n;
    let s_frac = scavenger_count as f32 / n;

    let mut entropy = 0.0f32;
    if h_frac > 0.0 {
        entropy -= h_frac * h_frac.log2();
    }
    if c_frac > 0.0 {
        entropy -= c_frac * c_frac.log2();
    }
    if s_frac > 0.0 {
        entropy -= s_frac * s_frac.log2();
    }

    let max_entropy = 3.0f32.log2(); // ~1.58 for 3 roles
    (entropy / max_entropy).clamp(0.0, 1.0)
}

/// Track metrics over time for trend analysis
#[derive(Clone, Debug)]
pub struct MetricsHistory {
    samples: Vec<PopulationMetrics>,
    max_samples: usize,
}

impl MetricsHistory {
    pub fn new(max_samples: usize) -> Self {
        Self {
            samples: Vec::with_capacity(max_samples),
            max_samples,
        }
    }

    pub fn record(&mut self, metrics: PopulationMetrics) {
        if self.samples.len() >= self.max_samples {
            self.samples.remove(0);
        }
        self.samples.push(metrics);
    }

    pub fn latest(&self) -> Option<&PopulationMetrics> {
        self.samples.last()
    }

    pub fn population_trend(&self) -> f32 {
        if self.samples.len() < 2 {
            return 0.0;
        }

        let recent = &self.samples[self.samples.len() - 1];
        let older = &self.samples[self.samples.len() / 2];

        if older.total_alive == 0 {
            return 0.0;
        }

        (recent.total_alive as f32 - older.total_alive as f32) / older.total_alive as f32
    }

    pub fn diversity_trend(&self) -> f32 {
        if self.samples.len() < 2 {
            return 0.0;
        }

        let recent = &self.samples[self.samples.len() - 1];
        let older = &self.samples[self.samples.len() / 2];

        recent.diversity - older.diversity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Genome;
    use glam::Vec2;

    #[test]
    fn test_population_metrics_empty() {
        let arena: BoidArena<100> = BoidArena::new();
        let metrics = PopulationMetrics::compute(&arena);

        assert_eq!(metrics.total_alive, 0);
        assert_eq!(metrics.diversity, 0.0);
    }

    #[test]
    fn test_population_metrics_monoculture() {
        let mut arena: BoidArena<100> = BoidArena::new();

        // All herbivores
        for i in 0..50 {
            let mut genes = Genome::default();
            genes.role = BoidRole::Herbivore;
            arena.spawn(Vec2::new(i as f32, 0.0), Vec2::ZERO, genes);
        }

        let metrics = PopulationMetrics::compute(&arena);

        assert_eq!(metrics.total_alive, 50);
        assert_eq!(metrics.herbivore_count, 50);
        assert_eq!(metrics.carnivore_count, 0);
        assert_eq!(metrics.scavenger_count, 0);
        assert!(
            metrics.diversity < 0.1,
            "Monoculture should have low diversity"
        );
    }

    #[test]
    fn test_population_metrics_balanced() {
        let mut arena: BoidArena<100> = BoidArena::new();

        // Equal distribution
        for i in 0..20 {
            let mut genes = Genome::default();
            genes.role = BoidRole::Herbivore;
            arena.spawn(Vec2::new(i as f32, 0.0), Vec2::ZERO, genes);
        }
        for i in 0..20 {
            let mut genes = Genome::default();
            genes.role = BoidRole::Carnivore;
            arena.spawn(Vec2::new(i as f32, 10.0), Vec2::ZERO, genes);
        }
        for i in 0..20 {
            let mut genes = Genome::default();
            genes.role = BoidRole::Scavenger;
            arena.spawn(Vec2::new(i as f32, 20.0), Vec2::ZERO, genes);
        }

        let metrics = PopulationMetrics::compute(&arena);

        assert_eq!(metrics.total_alive, 60);
        assert_eq!(metrics.herbivore_count, 20);
        assert_eq!(metrics.carnivore_count, 20);
        assert_eq!(metrics.scavenger_count, 20);
        assert!(
            metrics.diversity > 0.9,
            "Balanced population should have high diversity: {}",
            metrics.diversity
        );
    }

    #[test]
    fn test_dominant_role() {
        let mut arena: BoidArena<100> = BoidArena::new();

        for i in 0..30 {
            let mut genes = Genome::default();
            genes.role = BoidRole::Herbivore;
            arena.spawn(Vec2::new(i as f32, 0.0), Vec2::ZERO, genes);
        }
        for i in 0..10 {
            let mut genes = Genome::default();
            genes.role = BoidRole::Carnivore;
            arena.spawn(Vec2::new(i as f32, 10.0), Vec2::ZERO, genes);
        }

        let metrics = PopulationMetrics::compute(&arena);
        assert_eq!(metrics.dominant_role(), Some(BoidRole::Herbivore));
    }

    #[test]
    fn test_metrics_history() {
        let mut history = MetricsHistory::new(10);

        for i in 0..15 {
            let metrics = PopulationMetrics {
                total_alive: 100 + i * 10,
                ..Default::default()
            };
            history.record(metrics);
        }

        // Should only keep last 10
        assert_eq!(history.samples.len(), 10);

        // Latest should be the last one
        assert_eq!(history.latest().unwrap().total_alive, 100 + 14 * 10);
    }
}
