use std::collections::{
    VecDeque,
};
use std::fmt;


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Entity(u64);

impl Entity {
    #[inline]
    pub fn id(self) -> u64 {
        self.0
    }
}

impl fmt::Display for Entity {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct EntityAllocator {
    max_id: u64,
    available_entities: VecDeque<Entity>,
}

impl EntityAllocator {
    pub fn new() -> EntityAllocator {
        EntityAllocator {
            max_id: 0,
            available_entities: VecDeque::new(),
        }
    }

    pub fn allocate(&mut self) -> Entity {
        if !self.available_entities.is_empty() {
            // SAFETY: We know that the queue contains an element.
            self.available_entities.pop_front().unwrap()
        } else {
            let new_entity = Entity(self.max_id);
            self.max_id += 1;

            new_entity
        }
    }

    pub fn deallocate(&mut self, entity: Entity) {
        if entity.id() < self.max_id {
            // The entity has been allocated.
            self.available_entities.push_back(entity)
        }
    }
}

impl Default for EntityAllocator {
    fn default() -> EntityAllocator {
        EntityAllocator::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_allocate_deallocate_from_empty_allocator() {
        let mut allocator = EntityAllocator::new();
        let expected = allocator.allocate();
        allocator.deallocate(expected);
        let result = allocator.allocate();

        assert_eq!(result, expected);
    }
}

