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

/// Where all the data is grouped together.
struct World {
    entities: EntityLocationMap,
    entity_types: Vec<EntityTypeMap>,
    entity_allocator: EntityAllocator,
    components: ComponentSet,
}

impl World {
    pub fn new() -> World {
        todo!()
    }

    pub fn is_empty(&self) -> bool {
        todo!()
    }

    pub fn len(&self) -> usize {
        todo!()
    }

    pub fn contains(&self, entity: Entity) -> bool {
        todo!()
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


