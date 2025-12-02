//! Network metrics and analysis
//!
//! Provides tools for analyzing network properties and validating power law distributions.

use crate::powerlaw::{EdgeArena, NetworkArena};

/// Comprehensive network metrics
#[derive(Clone, Debug, Default)]
pub struct NetworkMetrics {
    pub node_count: usize,
    pub edge_count: usize,
    pub total_degree: u64,
    pub avg_degree: f32,
    pub max_degree: u32,
    pub min_degree: u32,
    pub alpha_estimate: f32,
    pub gini_coefficient: f32,
}

impl NetworkMetrics {
    /// Compute metrics from network. O(n) single-pass where possible.
    pub fn compute<const N: usize, const E: usize>(
        network: &NetworkArena<N>,
        edges: &EdgeArena<N, E>,
    ) -> Self {
        if network.alive_count == 0 {
            return Self::default();
        }

        let mut metrics = Self {
            node_count: network.alive_count,
            edge_count: edges.edge_count,
            total_degree: network.total_degree,
            min_degree: u32::MAX,
            ..Default::default()
        };

        // Single pass for degree stats
        for i in 0..N {
            if !network.alive[i] {
                continue;
            }
            let deg = network.degrees[i];
            if deg > metrics.max_degree {
                metrics.max_degree = deg;
            }
            if deg < metrics.min_degree {
                metrics.min_degree = deg;
            }
        }

        metrics.avg_degree = metrics.total_degree as f32 / metrics.node_count as f32;

        // Estimate power law exponent (Hill estimator)
        metrics.alpha_estimate = estimate_alpha(&network.degrees, &network.alive);

        // Compute Gini coefficient for resource distribution
        metrics.gini_coefficient = compute_gini(&network.resource, &network.alive);

        metrics
    }
}

/// Estimate power law exponent using Hill estimator
fn estimate_alpha(degrees: &[u32], alive: &[bool]) -> f32 {
    let mut valid: Vec<f32> = degrees
        .iter()
        .zip(alive.iter())
        .filter(|(_, &a)| a)
        .filter(|(&d, _)| d > 0)
        .map(|(&d, _)| d as f32)
        .collect();

    if valid.len() < 10 {
        return 0.0;
    }

    valid.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let x_min = valid[valid.len() / 10];

    let filtered: Vec<f32> = valid.iter().filter(|&&x| x >= x_min).copied().collect();

    if filtered.is_empty() {
        return 0.0;
    }

    let n = filtered.len() as f32;
    let log_sum: f32 = filtered.iter().map(|&x| (x / x_min).ln()).sum();

    1.0 + n / log_sum
}

/// Compute Gini coefficient (wealth inequality measure)
fn compute_gini(resources: &[f32], alive: &[bool]) -> f32 {
    let mut values: Vec<f32> = resources
        .iter()
        .zip(alive.iter())
        .filter(|(_, &a)| a)
        .map(|(&r, _)| r)
        .collect();

    if values.len() < 2 {
        return 0.0;
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let n = values.len() as f32;
    let sum: f32 = values.iter().sum();
    if sum <= 0.0 {
        return 0.0;
    }

    let weighted_sum: f32 = values
        .iter()
        .enumerate()
        .map(|(i, &x)| (i + 1) as f32 * x)
        .sum();

    ((2.0 * weighted_sum) / (n * sum) - (n + 1.0) / n).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gini_coefficient() {
        // Perfect equality
        let equal = vec![10.0, 10.0, 10.0, 10.0];
        let alive = vec![true, true, true, true];
        let gini = compute_gini(&equal, &alive);
        assert!(gini < 0.1, "Expected low Gini for equal distribution");

        // Perfect inequality
        let unequal = vec![0.0, 0.0, 0.0, 100.0];
        let gini = compute_gini(&unequal, &alive);
        assert!(gini > 0.7, "Expected high Gini for unequal distribution");
    }
}
