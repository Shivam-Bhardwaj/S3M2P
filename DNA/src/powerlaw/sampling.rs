//! Sampling algorithms for power law distributions
//!
//! Provides O(1) weighted sampling via Walker's Alias method and
//! pre-computed lookup tables for power law distributions.

use rand::Rng;

/// Walker's Alias method for O(1) weighted sampling
///
/// Used for preferential attachment when network changes infrequently.
/// Rebuild is O(n) but sampling is O(1).
pub struct AliasTable<const CAPACITY: usize> {
    /// Probability table (normalized to 1.0)
    prob: Vec<f32>,
    /// Alias indices
    alias: Vec<u16>,
    /// Number of valid entries
    pub size: usize,
    /// Flag indicating table needs rebuild
    pub dirty: bool,
}

impl<const CAPACITY: usize> AliasTable<CAPACITY> {
    pub fn new() -> Self {
        Self {
            prob: vec![0.0; CAPACITY],
            alias: vec![0; CAPACITY],
            size: 0,
            dirty: true,
        }
    }

    /// Sample using alias table. O(1) operation.
    ///
    /// Returns None if table is empty or dirty.
    #[inline]
    pub fn sample(&self, rng: &mut impl Rng) -> Option<usize> {
        if self.size == 0 || self.dirty {
            return None;
        }

        let i = rng.gen_range(0..self.size);
        let u: f32 = rng.gen();

        if u < self.prob[i] {
            Some(i)
        } else {
            Some(self.alias[i] as usize)
        }
    }

    /// Rebuild alias table from weights. O(n) operation.
    ///
    /// Implements Walker's alias method:
    /// 1. Normalize weights to sum to n
    /// 2. Split into small (<1) and large (>=1) buckets
    /// 3. Pair small with large to create alias table
    pub fn rebuild(&mut self, weights: &[f32], mask: &[bool]) {
        self.size = 0;
        let mut sum = 0.0f32;

        // Count active entries and sum weights
        for (i, (&weight, &active)) in weights.iter().zip(mask.iter()).enumerate() {
            if active && weight > 0.0 {
                sum += weight;
                self.size += 1;
            }
        }

        if self.size == 0 {
            self.dirty = false;
            return;
        }

        // Normalize to size
        let scale = self.size as f32 / sum;

        // Separate into small and large buckets
        let mut small = Vec::with_capacity(self.size);
        let mut large = Vec::with_capacity(self.size);

        let mut active_indices: Vec<usize> = Vec::with_capacity(self.size);
        for (i, (&weight, &active)) in weights.iter().zip(mask.iter()).enumerate() {
            if active && weight > 0.0 {
                let idx = active_indices.len();
                active_indices.push(i);
                let prob = weight * scale;
                self.prob[idx] = prob;

                if prob < 1.0 {
                    small.push(idx);
                } else {
                    large.push(idx);
                }
            }
        }

        // Build alias table
        while let (Some(s), Some(l)) = (small.pop(), large.last().copied()) {
            self.alias[s] = l as u16;

            self.prob[l] = (self.prob[l] + self.prob[s]) - 1.0;
            if self.prob[l] < 1.0 {
                large.pop();
                small.push(l);
            }
        }

        // Handle numerical errors
        for &l in &large {
            self.prob[l] = 1.0;
        }
        for &s in &small {
            self.prob[s] = 1.0;
        }

        self.dirty = false;
    }

    /// Mark table as dirty (needs rebuild)
    #[inline]
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

impl<const CAPACITY: usize> Default for AliasTable<CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

/// Power law sampler with pre-computed inverse CDF
///
/// For distribution P(x) ~ x^(-alpha), uses inverse transform:
/// x = x_min * (1 - u)^(-1/(alpha-1))
pub struct PowerLawSampler<const TABLE_SIZE: usize> {
    /// Pre-computed inverse CDF values
    inverse_cdf: Vec<f32>,
    /// Power law exponent (typically 2.0-3.0 for scale-free)
    pub alpha: f32,
    /// Minimum value
    pub x_min: f32,
    /// Maximum value
    pub x_max: f32,
}

impl<const TABLE_SIZE: usize> PowerLawSampler<TABLE_SIZE> {
    pub fn new(alpha: f32, x_min: f32, x_max: f32) -> Self {
        assert!(alpha > 1.0, "alpha must be > 1.0");
        assert!(x_min > 0.0, "x_min must be > 0.0");
        assert!(x_max > x_min, "x_max must be > x_min");

        let mut sampler = Self {
            inverse_cdf: vec![0.0; TABLE_SIZE],
            alpha,
            x_min,
            x_max,
        };
        sampler.build_table();
        sampler
    }

    fn build_table(&mut self) {
        let exp = -1.0 / (self.alpha - 1.0);

        for i in 0..TABLE_SIZE {
            let u = (i as f32 + 0.5) / TABLE_SIZE as f32;
            // Inverse CDF: x = x_min * (1 - u)^(-1/(alpha-1))
            let x = self.x_min * (1.0 - u).powf(exp);
            self.inverse_cdf[i] = x.min(self.x_max);
        }
    }

    /// Sample from power law distribution. O(1) operation.
    ///
    /// Uses table lookup with linear interpolation for smoother distribution.
    #[inline]
    pub fn sample(&self, rng: &mut impl Rng) -> f32 {
        let u: f32 = rng.gen();
        let idx_f = u * (TABLE_SIZE - 1) as f32;
        let idx = idx_f as usize;
        let frac = idx_f - idx as f32;

        // Linear interpolation
        let v0 = self.inverse_cdf[idx];
        let v1 = self.inverse_cdf[(idx + 1).min(TABLE_SIZE - 1)];
        v0 + frac * (v1 - v0)
    }

    /// PDF: f(x) = (alpha-1)/x_min * (x/x_min)^(-alpha)
    pub fn pdf(&self, x: f32) -> f32 {
        if x < self.x_min || x > self.x_max {
            return 0.0;
        }
        (self.alpha - 1.0) / self.x_min * (x / self.x_min).powf(-self.alpha)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_table() {
        let mut alias: AliasTable<10> = AliasTable::new();
        let mut rng = rand::thread_rng();

        // Weights: [10, 1, 1] -> should sample 0 most often
        let weights = vec![10.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let mask = vec![true, true, true, false, false, false, false, false, false, false];

        alias.rebuild(&weights, &mask);
        assert!(!alias.dirty);
        assert_eq!(alias.size, 3);

        // Sample many times
        let mut counts = vec![0usize; 3];
        for _ in 0..1200 {
            if let Some(idx) = alias.sample(&mut rng) {
                if idx < 3 {
                    counts[idx] += 1;
                }
            }
        }

        // Index 0 should dominate (should be ~83% = 10/12)
        assert!(
            counts[0] > 900,
            "Expected weighted sampling, got {:?}",
            counts
        );
    }

    #[test]
    fn test_power_law_sampler() {
        let sampler: PowerLawSampler<1024> = PowerLawSampler::new(2.5, 1.0, 1000.0);
        let mut rng = rand::thread_rng();

        // Sample many values
        let mut samples = Vec::new();
        for _ in 0..1000 {
            let x = sampler.sample(&mut rng);
            assert!(x >= 1.0 && x <= 1000.0);
            samples.push(x);
        }

        // Check that most samples are near x_min (characteristic of power law)
        let below_10 = samples.iter().filter(|&&x| x < 10.0).count();
        assert!(
            below_10 > 600,
            "Expected concentration near x_min, got {} below 10",
            below_10
        );
    }

    #[test]
    fn test_power_law_pdf() {
        let sampler: PowerLawSampler<256> = PowerLawSampler::new(2.5, 1.0, 100.0);

        // PDF should be higher at smaller x values
        let pdf_1 = sampler.pdf(1.0);
        let pdf_10 = sampler.pdf(10.0);
        let pdf_100 = sampler.pdf(100.0);

        assert!(pdf_1 > pdf_10);
        assert!(pdf_10 > pdf_100);
        assert!(pdf_1 > 0.0);
    }
}
