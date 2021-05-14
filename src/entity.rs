use std::collections::{
    VecDeque,
};
use std::fmt;


#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Entity(u64);


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
        self.available_entities.push_back(entity)
    }
}

impl Default for EntityAllocator {
    fn default() -> EntityAllocator {
        EntityAllocator::new()
    }
}

