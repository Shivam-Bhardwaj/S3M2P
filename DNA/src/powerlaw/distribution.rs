//! Power law distributions
//!
//! Provides Pareto and Zipf distributions for resource allocation and ranked data.

use rand::Rng;

/// Pareto distribution: P(x) ~ x^(-alpha) for x >= x_min
#[derive(Clone, Copy, Debug)]
pub struct Pareto {
    pub alpha: f32,
    pub x_min: f32,
}

impl Pareto {
    pub fn new(alpha: f32, x_min: f32) -> Self {
        assert!(alpha > 1.0, "alpha must be > 1.0");
        assert!(x_min > 0.0, "x_min must be > 0.0");
        Self { alpha, x_min }
    }

    /// Sample using inverse transform: x = x_min * (1 - u)^(-1/(alpha-1))
    pub fn sample(&self, rng: &mut impl Rng) -> f32 {
        let u: f32 = rng.gen();
        self.x_min * (1.0 - u).powf(-1.0 / (self.alpha - 1.0))
    }

    pub fn pdf(&self, x: f32) -> f32 {
        if x < self.x_min {
            return 0.0;
        }
        (self.alpha - 1.0) / self.x_min * (x / self.x_min).powf(-self.alpha)
    }
}

/// Zipf's law: frequency ~ 1/rank^s
#[derive(Clone, Debug)]
pub struct Zipf {
    pub s: f64,
    harmonic_cache: Vec<f64>,
}

impl Zipf {
    pub fn new(s: f64, max_rank: usize) -> Self {
        let mut harmonic_cache = Vec::with_capacity(max_rank + 1);
        harmonic_cache.push(0.0);
        let mut h = 0.0;
        for k in 1..=max_rank {
            h += 1.0 / (k as f64).powf(s);
            harmonic_cache.push(h);
        }
        Self { s, harmonic_cache }
    }

    /// Sample rank using binary search
    pub fn sample_rank(&self, rng: &mut impl Rng) -> usize {
        let n = self.harmonic_cache.len() - 1;
        if n == 0 {
            return 1;
        }

        let h_n = self.harmonic_cache[n];
        let u: f64 = rng.gen();
        let target = u * h_n;

        // Binary search
        let mut lo = 1;
        let mut hi = n;
        while lo < hi {
            let mid = (lo + hi) / 2;
            if self.harmonic_cache[mid] < target {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        lo
    }

    pub fn probability(&self, rank: usize) -> f64 {
        let max_rank = self.harmonic_cache.len() - 1;
        if rank == 0 || rank > max_rank {
            return 0.0;
        }
        let h_n = self.harmonic_cache[max_rank];
        1.0 / ((rank as f64).powf(self.s) * h_n)
    }
}
