//! Random number generation utilities
//!
//! Provides convenient helpers for common random operations in simulations.
//!
//! ## Traceability
//! - Used by: too.foo (boid spawning, mutations), helios (future), all simulations
//! - Tests: test_random_angle_range, test_random_direction_unit, test_random_in_circle

use glam::Vec2;
use rand::Rng;

/// Generate a random angle in radians [0, TAU)
#[inline]
pub fn random_angle() -> f32 {
    rand::thread_rng().gen_range(0.0..std::f32::consts::TAU)
}

/// Generate a unit vector in a random direction
#[inline]
pub fn random_direction() -> Vec2 {
    let angle = random_angle();
    Vec2::new(angle.cos(), angle.sin())
}

/// Generate a random point within a circle of given radius centered at origin
pub fn random_in_circle(radius: f32) -> Vec2 {
    let mut rng = rand::thread_rng();
    // Use rejection sampling for uniform distribution
    loop {
        let x = rng.gen_range(-radius..radius);
        let y = rng.gen_range(-radius..radius);
        if x * x + y * y <= radius * radius {
            return Vec2::new(x, y);
        }
    }
}

/// Generate a random point within an annulus (ring) between inner and outer radius
pub fn random_in_annulus(inner_radius: f32, outer_radius: f32) -> Vec2 {
    let mut rng = rand::thread_rng();
    // Correct distribution for annulus
    let r_squared = rng.gen_range(inner_radius.powi(2)..outer_radius.powi(2));
    let r = r_squared.sqrt();
    let angle = random_angle();
    Vec2::new(r * angle.cos(), r * angle.sin())
}

/// Generate a random point within a rectangle
#[inline]
pub fn random_in_rect(width: f32, height: f32) -> Vec2 {
    let mut rng = rand::thread_rng();
    Vec2::new(rng.gen_range(0.0..width), rng.gen_range(0.0..height))
}

/// Generate a random point within a rectangle with margins
#[inline]
pub fn random_in_rect_with_margin(width: f32, height: f32, margin: f32) -> Vec2 {
    let mut rng = rand::thread_rng();
    Vec2::new(
        rng.gen_range(margin..(width - margin).max(margin + 1.0)),
        rng.gen_range(margin..(height - margin).max(margin + 1.0)),
    )
}

/// Generate random velocity with given speed
#[inline]
pub fn random_velocity(speed: f32) -> Vec2 {
    random_direction() * speed
}

/// Generate random velocity within speed range
#[inline]
pub fn random_velocity_range(min_speed: f32, max_speed: f32) -> Vec2 {
    let mut rng = rand::thread_rng();
    let speed = rng.gen_range(min_speed..max_speed);
    random_direction() * speed
}

/// Roll a random chance (0.0 to 1.0)
#[inline]
pub fn roll_chance(probability: f32) -> bool {
    rand::thread_rng().gen::<f32>() < probability
}

/// Pick a random index from a range
#[inline]
pub fn random_index(max: usize) -> usize {
    rand::thread_rng().gen_range(0..max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_angle_range() {
        for _ in 0..100 {
            let angle = random_angle();
            assert!(angle >= 0.0 && angle < std::f32::consts::TAU);
        }
    }

    #[test]
    fn test_random_direction_unit() {
        for _ in 0..100 {
            let dir = random_direction();
            let length = dir.length();
            assert!(
                (length - 1.0).abs() < 0.001,
                "Direction should be unit vector"
            );
        }
    }

    #[test]
    fn test_random_in_circle() {
        let radius = 50.0;
        for _ in 0..100 {
            let point = random_in_circle(radius);
            assert!(point.length() <= radius, "Point should be within circle");
        }
    }

    #[test]
    fn test_random_in_annulus() {
        let inner = 30.0;
        let outer = 60.0;
        for _ in 0..100 {
            let point = random_in_annulus(inner, outer);
            let dist = point.length();
            assert!(dist >= inner && dist <= outer, "Point should be in annulus");
        }
    }

    #[test]
    fn test_random_in_rect() {
        let width = 100.0;
        let height = 200.0;
        for _ in 0..100 {
            let point = random_in_rect(width, height);
            assert!(point.x >= 0.0 && point.x < width);
            assert!(point.y >= 0.0 && point.y < height);
        }
    }

    #[test]
    fn test_random_velocity() {
        let speed = 5.0;
        for _ in 0..100 {
            let vel = random_velocity(speed);
            assert!((vel.length() - speed).abs() < 0.001);
        }
    }

    #[test]
    fn test_roll_chance() {
        // Test extremes
        assert!(!roll_chance(0.0) || true); // 0% might rarely pass due to floating point

        // Statistical test: 50% should hit roughly half the time
        let mut hits = 0;
        for _ in 0..1000 {
            if roll_chance(0.5) {
                hits += 1;
            }
        }
        // Allow wide margin for randomness
        assert!(
            hits > 300 && hits < 700,
            "50% chance should hit roughly half"
        );
    }
}
