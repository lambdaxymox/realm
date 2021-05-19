use crate::component::{
    ComponentTypeIndex,
};
use crate::entity::{
    Entity,
    EntityAllocator,
};
use crate::storage::{
    UnsafeComponentStorage,
    EntityLocationMap,
    EntityTypeMap,
    EntityType,
    EntityTypeIndex,
};
use std::collections::{
    HashMap,
};


/// where the components live in a world.
struct ComponentSet {
    data: HashMap<ComponentTypeIndex, Box<dyn UnsafeComponentStorage>>,
}

impl ComponentSet {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}


/// Where all the data is grouped together.
struct World {
    entities: EntityLocationMap,
    entity_types: Vec<EntityTypeMap>,
    entity_allocator: EntityAllocator,
    components: ComponentSet,
}

impl World {
    pub fn new() -> World {
        World {
            entities: EntityLocationMap::new(),
            entity_types: Vec::new(),
            entity_allocator: EntityAllocator::new(),
            components: ComponentSet::new()
        }
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.entities.contains(entity)
    }

    pub fn remove(&mut self, entity: Entity) -> bool {
        todo!()
    }

    pub fn clear(&mut self) {
        todo!()
    }

    pub fn components(&self) -> &ComponentSet {
        todo!()
    }

    pub fn components_mut(&mut self) -> &mut ComponentSet {
        todo!()
    }

    pub fn entity_types(&self) -> &[EntityType] {
        todo!()
    }

    fn create_new_entity_type(&mut self, entity_type: EntityTypeIndex) -> EntityTypeMap {
        todo!()
    }
}


