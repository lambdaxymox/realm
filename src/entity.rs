#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Entity(u64);


pub struct EntityAllocator {
    max_id: u64,
    available_entities: Vec<Entity>,
}

impl EntityAllocator {
    pub fn new() -> EntityAllocator {
        EntityAllocator {
            max_id: 0,
            available_entities: vec![],
        }
    }
}

impl Default for EntityAllocator {
    fn default() -> EntityAllocator {
        EntityAllocator::new()
    }
}

