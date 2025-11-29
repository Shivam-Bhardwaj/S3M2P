//! Interaction effects for entity-environment and entity-entity interactions
//!
//! Provides a unified system for handling interactions between:
//! - Boids and food sources
//! - Boids and fungal networks
//! - Boids and environmental hazards
//!
//! ## Traceability
//! - Used by: too.foo (fungal network), future simulations
//! - Tests: test_interaction_result_variants

/// Result of an interaction between an entity and its environment or another entity
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InteractionResult {
    /// No effect from this interaction
    None,
    /// Entity gains energy/resources
    Nutrient(f32),
    /// Entity takes damage/loses energy
    Damage(f32),
    /// Entity is immediately killed/destroyed
    Death,
}

impl InteractionResult {
    /// Check if this result has any effect
    #[inline]
    pub fn has_effect(&self) -> bool {
        !matches!(self, InteractionResult::None)
    }

    /// Get the energy change from this result (positive = gain, negative = loss)
    #[inline]
    pub fn energy_delta(&self) -> f32 {
        match self {
            InteractionResult::None => 0.0,
            InteractionResult::Nutrient(amt) => *amt,
            InteractionResult::Damage(amt) => -amt,
            InteractionResult::Death => f32::NEG_INFINITY,
        }
    }

    /// Check if this result is fatal
    #[inline]
    pub fn is_fatal(&self) -> bool {
        matches!(self, InteractionResult::Death)
    }

    /// Check if this result is beneficial
    #[inline]
    pub fn is_beneficial(&self) -> bool {
        matches!(self, InteractionResult::Nutrient(_))
    }

    /// Check if this result is harmful
    #[inline]
    pub fn is_harmful(&self) -> bool {
        matches!(
            self,
            InteractionResult::Damage(_) | InteractionResult::Death
        )
    }
}

/// Trait for entities that can interact with others
pub trait Interactable {
    /// Get the interaction result for an entity at the given position
    fn get_interaction(&self, entity_pos: glam::Vec2) -> InteractionResult;
}

/// Batch process interactions and collect results
/// Returns a vector of (entity_index, InteractionResult) pairs
pub fn process_interactions<T: Interactable>(
    positions: &[glam::Vec2],
    alive: &[bool],
    interactables: &[T],
) -> Vec<(usize, InteractionResult)> {
    let mut results = Vec::new();

    for (idx, &pos) in positions.iter().enumerate() {
        if !alive[idx] {
            continue;
        }

        for interactable in interactables {
            let result = interactable.get_interaction(pos);
            if result.has_effect() {
                results.push((idx, result));
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interaction_result_variants() {
        // None
        let none = InteractionResult::None;
        assert!(!none.has_effect());
        assert_eq!(none.energy_delta(), 0.0);
        assert!(!none.is_fatal());
        assert!(!none.is_beneficial());
        assert!(!none.is_harmful());

        // Nutrient
        let nutrient = InteractionResult::Nutrient(10.0);
        assert!(nutrient.has_effect());
        assert_eq!(nutrient.energy_delta(), 10.0);
        assert!(!nutrient.is_fatal());
        assert!(nutrient.is_beneficial());
        assert!(!nutrient.is_harmful());

        // Damage
        let damage = InteractionResult::Damage(5.0);
        assert!(damage.has_effect());
        assert_eq!(damage.energy_delta(), -5.0);
        assert!(!damage.is_fatal());
        assert!(!damage.is_beneficial());
        assert!(damage.is_harmful());

        // Death
        let death = InteractionResult::Death;
        assert!(death.has_effect());
        assert!(death.energy_delta().is_infinite() && death.energy_delta() < 0.0);
        assert!(death.is_fatal());
        assert!(!death.is_beneficial());
        assert!(death.is_harmful());
    }

    #[test]
    fn test_batch_processing() {
        use glam::Vec2;

        // Simple test interactable
        struct TestZone {
            center: Vec2,
            radius: f32,
            effect: InteractionResult,
        }

        impl Interactable for TestZone {
            fn get_interaction(&self, entity_pos: Vec2) -> InteractionResult {
                if entity_pos.distance(self.center) < self.radius {
                    self.effect
                } else {
                    InteractionResult::None
                }
            }
        }

        let positions = vec![
            Vec2::new(100.0, 100.0), // Inside zone
            Vec2::new(500.0, 500.0), // Outside zone
            Vec2::new(110.0, 100.0), // Inside zone
        ];
        let alive = vec![true, true, true];

        let zones = vec![TestZone {
            center: Vec2::new(100.0, 100.0),
            radius: 50.0,
            effect: InteractionResult::Damage(5.0),
        }];

        let results = process_interactions(&positions, &alive, &zones);

        // Should have 2 results (indices 0 and 2 are inside)
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 0);
        assert_eq!(results[1].0, 2);
    }
}
