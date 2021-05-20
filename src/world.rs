use crate::component::{
    Component,
    ComponentTypeIndex,
};
use crate::entity::{
    Entity,
    EntityAllocator,
};
use crate::storage::{
    UnknownComponentStorage,
    EntityLocationMap,
    EntityTypeMap,
    EntityType,
    EntityLocation,
    StoreComponentsIn,
    ComponentStorage,
};
use std::collections::{
    HashMap,
};


/// where the components live in a world.
struct ComponentMap {
    data: HashMap<ComponentTypeIndex, Box<dyn UnknownComponentStorage>>,
}

impl ComponentMap {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    fn get(&self, component_type: ComponentTypeIndex) -> Option<&dyn UnknownComponentStorage> {
        self.data.get(&component_type).map(|cell| cell.as_ref())
    }

    fn get_view<T: Component + StoreComponentsIn<T>>(&self) -> Option<&T::Storage> {
        let type_id = ComponentTypeIndex::of::<T>();
        None
    }
}

pub struct Entry<'a> {
    location: EntityLocation,
    world: &'a mut World,
}

impl<'a> Entry<'a> {
    fn new(location: EntityLocation, world: &'a mut World) -> Self {
        Self {
            location: location,
            world: world,
        }
    }

    pub fn entity_type(&self) -> &EntityType {
        &self.world.entity_types()[self.location.entity_type().id()]
    }

    pub fn location(&self) -> EntityLocation {
        self.location
    }

    pub fn get_component<T: Component + StoreComponentsIn<T>>(&self) -> Result<&T, ()> {
        let entity_type = self.location.entity_type();
        let component = self.location.component();
        self.world
            .components()
            .get_view::<T>()
            .and_then(move |storage| storage.get(entity_type))
            .and_then(move |view| view.into_slice().get(component.id()))
            .ok_or_else(|| {})
    }
}

/// Where all the data is grouped together.
struct World {
    entities: EntityLocationMap,
    entity_types: Vec<EntityTypeMap>,
    entity_allocator: EntityAllocator,
    components: ComponentMap,
}

impl World {
    pub fn new() -> World {
        World {
            entities: EntityLocationMap::new(),
            entity_types: Vec::new(),
            entity_allocator: EntityAllocator::new(),
            components: ComponentMap::new()
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
        if let Some(location) = self.entities.remove(entity) {
            self.remove_at_location(location);

            true
        } else {
            false
        }
    }

    pub fn remove_at_location(&mut self, location: EntityLocation) {
        todo!()
    }

    pub fn clear(&mut self) {
        todo!()
    }

    pub fn components(&self) -> &ComponentMap {
        todo!()
    }

    pub fn components_mut(&mut self) -> &mut ComponentMap {
        todo!()
    }

    pub fn entity_types(&self) -> &[EntityType] {
        todo!()
    }

    fn create_new_entity_type(&mut self, entity_type: EntityType) -> EntityTypeMap {
        todo!()
    }
}


