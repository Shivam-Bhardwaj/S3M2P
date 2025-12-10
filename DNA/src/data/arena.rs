//! ═══════════════════════════════════════════════════════════════════════════════
//! FILE: arena.rs | DNA/src/data/arena.rs
//! PURPOSE: Generic fixed-capacity arena allocator with generational indices
//! MODIFIED: 2025-12-09
//! LAYER: DNA (foundation)
//! ═══════════════════════════════════════════════════════════════════════════════

//!
//! PURPOSE: Generic fixed-capacity arena allocator with generational indices
//!
//! LAYER: DNA → DATA
//!
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │ DATA DEFINED                                                                │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │ Arena<T, CAP>     Fixed-capacity arena with O(1) spawn/kill                 │
//! │ Handle            Generational index for safe entity references             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//!
//! ┌─────────────────────────────────────────────────────────────────────────────┐
//! │ DATA FLOW                                                                   │
//! ├─────────────────────────────────────────────────────────────────────────────┤
//! │ CONSUMES:  T (entity data)                                                  │
//! │ PRODUCES:  Handle (entity reference), &T/&mut T (entity access)             │
//! └─────────────────────────────────────────────────────────────────────────────┘
//!
//! DEPENDS ON:
//!   • None (pure data structure)
//!
//! USED BY:
//!   • Future: Particle systems, entity management, object pools
//!
//! ALGORITHM: Generational arena pattern with free list
//!   - Pre-allocated storage (no dynamic allocation)
//!   - O(1) spawn via free list
//!   - O(1) kill via tombstone + free list
//!   - Generation counter prevents use-after-free
//!
//! REFERENCE: https://github.com/fitzgen/generational-arena
//!
//! ═══════════════════════════════════════════════════════════════════════════════

// ─────────────────────────────────────────────────────────────────────────────────
// CODE BELOW - Optimized for ML development
// ─────────────────────────────────────────────────────────────────────────────────

/// Generational index handle for safe entity references
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Handle {
    index: u32,
    generation: u32,
}

/// Entry in the arena - either occupied with data or free (next free index + generation)
enum Entry<T> {
    Occupied {
        generation: u32,
        value: T,
    },
    Free {
        next_free: Option<u32>,
        next_generation: u32,
    },
}

/// Generic fixed-capacity arena allocator
///
/// Provides O(1) spawn/kill with generational indices to prevent use-after-free.
/// Pre-allocates all storage upfront (no dynamic allocation during use).
pub struct Arena<T, const CAPACITY: usize> {
    entries: Vec<Entry<T>>,
    free_list_head: Option<u32>,
    len: usize,
}

impl<T, const CAPACITY: usize> Arena<T, CAPACITY> {
    /// Create a new empty arena
    pub fn new() -> Self {
        let mut entries = Vec::with_capacity(CAPACITY);

        // Initialize free list
        for i in 0..CAPACITY {
            entries.push(Entry::Free {
                next_free: if i + 1 < CAPACITY {
                    Some((i + 1) as u32)
                } else {
                    None
                },
                next_generation: 0, // First generation is 0
            });
        }

        Self {
            entries,
            free_list_head: Some(0),
            len: 0,
        }
    }

    /// Spawn a new entity, returning its handle
    ///
    /// Returns None if arena is full.
    pub fn spawn(&mut self, value: T) -> Option<Handle> {
        let index = self.free_list_head?;

        match std::mem::replace(
            &mut self.entries[index as usize],
            Entry::Free {
                next_free: None,
                next_generation: 0,
            },
        ) {
            Entry::Free {
                next_free,
                next_generation,
            } => {
                // Use the generation from the free entry
                let generation = next_generation;

                self.entries[index as usize] = Entry::Occupied { generation, value };
                self.free_list_head = next_free;
                self.len += 1;

                Some(Handle { index, generation })
            }
            Entry::Occupied { .. } => {
                unreachable!("Free list points to occupied entry")
            }
        }
    }

    /// Kill an entity by handle
    ///
    /// Returns true if entity was killed, false if handle was invalid.
    pub fn kill(&mut self, handle: Handle) -> bool {
        match std::mem::replace(
            &mut self.entries[handle.index as usize],
            Entry::Free {
                next_free: None,
                next_generation: 0,
            },
        ) {
            Entry::Occupied { generation, .. } if generation == handle.generation => {
                // Entry was occupied with correct generation - kill it
                // Next spawn in this slot will use incremented generation
                let next_generation = generation.wrapping_add(1);

                self.entries[handle.index as usize] = Entry::Free {
                    next_free: self.free_list_head,
                    next_generation,
                };
                self.free_list_head = Some(handle.index);
                self.len -= 1;

                true
            }
            other => {
                // Wrong generation or already free - restore original state
                self.entries[handle.index as usize] = other;
                false
            }
        }
    }

    /// Get immutable reference to entity by handle
    pub fn get(&self, handle: Handle) -> Option<&T> {
        match &self.entries[handle.index as usize] {
            Entry::Occupied { generation, value } if *generation == handle.generation => {
                Some(value)
            }
            _ => None,
        }
    }

    /// Get mutable reference to entity by handle
    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut T> {
        match &mut self.entries[handle.index as usize] {
            Entry::Occupied { generation, value } if *generation == handle.generation => {
                Some(value)
            }
            _ => None,
        }
    }

    /// Number of alive entities
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if arena is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Iterate over all alive entities
    pub fn iter(&self) -> impl Iterator<Item = (Handle, &T)> {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| match entry {
                Entry::Occupied { generation, value } => Some((
                    Handle {
                        index: index as u32,
                        generation: *generation,
                    },
                    value,
                )),
                Entry::Free { .. } => None,
            })
    }

    /// Iterate mutably over all alive entities
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Handle, &mut T)> {
        self.entries
            .iter_mut()
            .enumerate()
            .filter_map(|(index, entry)| match entry {
                Entry::Occupied { generation, value } => Some((
                    Handle {
                        index: index as u32,
                        generation: *generation,
                    },
                    value,
                )),
                Entry::Free { .. } => None,
            })
    }
}

impl<T, const CAPACITY: usize> Default for Arena<T, CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_and_get() {
        let mut arena: Arena<i32, 10> = Arena::new();

        let h1 = arena.spawn(42).unwrap();
        let h2 = arena.spawn(99).unwrap();

        assert_eq!(arena.len(), 2);
        assert_eq!(*arena.get(h1).unwrap(), 42);
        assert_eq!(*arena.get(h2).unwrap(), 99);
    }

    #[test]
    fn test_kill_and_reuse() {
        let mut arena: Arena<i32, 10> = Arena::new();

        let h1 = arena.spawn(42).unwrap();
        assert!(arena.kill(h1));
        assert_eq!(arena.len(), 0);

        // Old handle should be invalid
        assert!(arena.get(h1).is_none());

        // Slot can be reused
        let h2 = arena.spawn(99).unwrap();
        assert_eq!(arena.len(), 1);
        assert_eq!(*arena.get(h2).unwrap(), 99);
    }

    #[test]
    fn test_generational_safety() {
        let mut arena: Arena<i32, 10> = Arena::new();

        let h1 = arena.spawn(42).unwrap();
        let old_handle = h1;

        arena.kill(h1);

        // Respawn in same slot
        let h2 = arena.spawn(99).unwrap();

        // Old handle should not access new value (different generation)
        assert!(arena.get(old_handle).is_none());
        assert_eq!(*arena.get(h2).unwrap(), 99);
    }

    #[test]
    fn test_capacity() {
        let mut arena: Arena<i32, 3> = Arena::new();

        arena.spawn(1).unwrap();
        arena.spawn(2).unwrap();
        arena.spawn(3).unwrap();

        // Arena is full
        assert!(arena.spawn(4).is_none());
    }

    #[test]
    fn test_iter() {
        let mut arena: Arena<i32, 10> = Arena::new();

        arena.spawn(10);
        arena.spawn(20);
        arena.spawn(30);

        let values: Vec<i32> = arena.iter().map(|(_, &v)| v).collect();
        assert_eq!(values, vec![10, 20, 30]);
    }
}
