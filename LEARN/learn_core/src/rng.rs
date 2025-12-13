//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: rng.rs | LEARN/learn_core/src/rng.rs
//! PURPOSE: Deterministic seeded random number generator (LCG)
//! MODIFIED: 2025-12-11
//! LAYER: LEARN → learn_core
//! ═══════════════════════════════════════════════════════════════════════════════
//!
//! Uses a Linear Congruential Generator (LCG) for reproducible randomness.
//! Same seed always produces same sequence - critical for educational simulations.

/// Deterministic random number generator using LCG algorithm
///
/// Uses the same constants as Numerical Recipes LCG.
/// Produces identical sequences given the same seed.
#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
}

impl Default for Rng {
    fn default() -> Self {
        Self::new(42)
    }
}

impl Rng {
    // LCG constants from Numerical Recipes
    const A: u64 = 6364136223846793005;
    const C: u64 = 1442695040888963407;

    /// Create a new RNG with the given seed
    #[inline]
    pub fn new(seed: u64) -> Self {
        // Mix the seed to avoid poor sequences from simple seeds
        let state = seed.wrapping_mul(Self::A).wrapping_add(Self::C);
        Self { state }
    }

    /// Get the next random u64
    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(Self::A).wrapping_add(Self::C);
        self.state
    }

    /// Get the next random u32
    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    /// Get a random f32 in [0, 1)
    #[inline]
    pub fn next_f32(&mut self) -> f32 {
        // Use top 24 bits for better distribution
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Get a random f64 in [0, 1)
    #[inline]
    pub fn next_f64(&mut self) -> f64 {
        // Use top 53 bits for full f64 precision
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Get a random f32 in [min, max)
    #[inline]
    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }

    /// Get a random f64 in [min, max)
    #[inline]
    pub fn range_f64(&mut self, min: f64, max: f64) -> f64 {
        min + self.next_f64() * (max - min)
    }

    /// Get a random integer in [min, max) (exclusive upper bound)
    #[inline]
    pub fn range_int(&mut self, min: i32, max: i32) -> i32 {
        if min >= max {
            return min;
        }
        let range = (max - min) as u64;
        min + (self.next_u64() % range) as i32
    }

    /// Get a random boolean with 50% probability
    #[inline]
    pub fn next_bool(&mut self) -> bool {
        self.next_u64() & 1 == 0
    }

    /// Get a random boolean with given probability of true
    #[inline]
    pub fn chance(&mut self, probability: f32) -> bool {
        self.next_f32() < probability
    }

    /// Generate a random value from standard normal distribution (mean=0, std=1)
    /// Uses Box-Muller transform
    pub fn normal(&mut self) -> f32 {
        let u1 = self.next_f32().max(1e-10); // Avoid log(0)
        let u2 = self.next_f32();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos()
    }

    /// Generate a random value from normal distribution with given mean and std
    #[inline]
    pub fn normal_with(&mut self, mean: f32, std: f32) -> f32 {
        mean + self.normal() * std
    }

    /// Shuffle a slice in place using Fisher-Yates algorithm
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        let len = slice.len();
        for i in (1..len).rev() {
            let j = (self.next_u64() as usize) % (i + 1);
            slice.swap(i, j);
        }
    }

    /// Pick a random element from a slice
    pub fn pick<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T> {
        if slice.is_empty() {
            None
        } else {
            let idx = (self.next_u64() as usize) % slice.len();
            Some(&slice[idx])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic() {
        let mut rng1 = Rng::new(42);
        let mut rng2 = Rng::new(42);

        for _ in 0..100 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
        }
    }

    #[test]
    fn test_different_seeds() {
        let mut rng1 = Rng::new(42);
        let mut rng2 = Rng::new(43);

        // Different seeds should produce different sequences
        let mut same = true;
        for _ in 0..10 {
            if rng1.next_u64() != rng2.next_u64() {
                same = false;
                break;
            }
        }
        assert!(!same);
    }

    #[test]
    fn test_next_f32_range() {
        let mut rng = Rng::new(12345);
        for _ in 0..1000 {
            let v = rng.next_f32();
            assert!(v >= 0.0 && v < 1.0, "Value out of range: {}", v);
        }
    }

    #[test]
    fn test_range() {
        let mut rng = Rng::new(54321);
        for _ in 0..1000 {
            let v = rng.range(-5.0, 10.0);
            assert!(v >= -5.0 && v < 10.0, "Value out of range: {}", v);
        }
    }

    #[test]
    fn test_range_int() {
        let mut rng = Rng::new(99999);
        for _ in 0..1000 {
            let v = rng.range_int(0, 10);
            assert!(v >= 0 && v < 10, "Value out of range: {}", v);
        }
    }

    #[test]
    fn test_normal_distribution() {
        let mut rng = Rng::new(777);
        let mut sum = 0.0;
        let n = 10000;

        for _ in 0..n {
            sum += rng.normal();
        }

        let mean = sum / n as f32;
        // Mean should be close to 0 for standard normal
        assert!(
            mean.abs() < 0.1,
            "Mean too far from 0: {}",
            mean
        );
    }

    #[test]
    fn test_chance() {
        let mut rng = Rng::new(123);
        let n = 10000;
        let mut count = 0;

        for _ in 0..n {
            if rng.chance(0.3) {
                count += 1;
            }
        }

        let ratio = count as f32 / n as f32;
        // Should be close to 0.3
        assert!(
            (ratio - 0.3).abs() < 0.05,
            "Ratio too far from 0.3: {}",
            ratio
        );
    }

    #[test]
    fn test_shuffle() {
        let mut rng = Rng::new(555);
        let mut arr = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let original = arr;

        rng.shuffle(&mut arr);

        // Should contain same elements
        let mut sorted = arr;
        sorted.sort();
        assert_eq!(sorted, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        // Should be different from original (very high probability)
        assert_ne!(arr, original);
    }

    #[test]
    fn test_pick() {
        let mut rng = Rng::new(888);
        let items = [1, 2, 3, 4, 5];

        for _ in 0..100 {
            let picked = rng.pick(&items);
            assert!(picked.is_some());
            assert!(items.contains(picked.unwrap()));
        }

        let empty: [i32; 0] = [];
        assert!(rng.pick(&empty).is_none());
    }

    #[test]
    fn test_distribution_uniformity() {
        // Chi-squared test for uniformity
        let mut rng = Rng::new(12345);
        let n_buckets = 10;
        let n_samples = 10000;
        let expected = n_samples / n_buckets;
        let mut buckets = vec![0usize; n_buckets];

        for _ in 0..n_samples {
            let v = rng.next_f32();
            let bucket = (v * n_buckets as f32) as usize;
            let bucket = bucket.min(n_buckets - 1);
            buckets[bucket] += 1;
        }

        // Each bucket should be within 20% of expected
        for (i, &count) in buckets.iter().enumerate() {
            let ratio = count as f32 / expected as f32;
            assert!(
                ratio > 0.8 && ratio < 1.2,
                "Bucket {} has {} samples, expected ~{} (ratio: {})",
                i,
                count,
                expected,
                ratio
            );
        }
    }
}
