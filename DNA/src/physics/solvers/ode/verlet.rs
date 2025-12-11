//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: verlet.rs | DNA/src/physics/solvers/ode/verlet.rs
//! PURPOSE: Velocity Verlet integrator (symplectic, energy-conserving)
//! MODIFIED: 2025-12-09
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════

//!
//! PURPOSE: Velocity Verlet integrator (symplectic, energy-conserving)
//!
//! LAYER: DNA → PHYSICS → SOLVERS → ODE
//!
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │ ALGORITHM                                                                   │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │ Velocity Verlet for x''(t) = a(x):                                          │
//! │                                                                             │
//! │   x_{n+1} = x_n + v_n·dt + ½a_n·dt²                                         │
//! │   a_{n+1} = a(x_{n+1})                                                      │
//! │   v_{n+1} = v_n + ½(a_n + a_{n+1})·dt                                       │
//! │                                                                             │
//! │ Properties: Symplectic (conserves energy), time-reversible                  │
//! │ Best for: Conservative systems (orbital mechanics, molecular dynamics)      │
//! └─────────────────────────────────────────────────────────────────────────────┘
//!
//! REFERENCE: https://en.wikipedia.org/wiki/Verlet_integration
//!
//! ═══════════════════════════════════════════════════════════════════════════════

use glam::Vec3;

/// Velocity Verlet step for 3D particle
pub fn verlet_step<F>(
    position: Vec3,
    velocity: Vec3,
    acceleration: Vec3,
    dt: f32,
    accel_fn: F,
) -> (Vec3, Vec3, Vec3)
where
    F: Fn(Vec3) -> Vec3,
{
    // Update position
    let new_position = position + velocity * dt + 0.5 * acceleration * dt * dt;

    // Compute new acceleration
    let new_acceleration = accel_fn(new_position);

    // Update velocity (average of old and new acceleration)
    let new_velocity = velocity + 0.5 * (acceleration + new_acceleration) * dt;

    (new_position, new_velocity, new_acceleration)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harmonic_oscillator() {
        // Simple harmonic oscillator: a = -k·x, k = 1
        let mut pos = Vec3::new(1.0, 0.0, 0.0);
        let mut vel = Vec3::ZERO;
        let mut acc = Vec3::new(-1.0, 0.0, 0.0);

        let dt = 0.01;
        for _ in 0..628 {
            // ~2π seconds
            let (p, v, a) = verlet_step(pos, vel, acc, dt, |x| -x);
            pos = p;
            vel = v;
            acc = a;
        }

        // Should return close to initial position (conservative system)
        assert!((pos.x - 1.0).abs() < 0.1);
    }
}
