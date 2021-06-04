use crate::storage::{
    EntityType,
    EntityLocation,
};
use crate::world::{
    World,
};


pub struct Entry<'a> {
    key: EntityLocation,
    world: &'a mut World,
}

impl<'a> Entry<'a> {
    pub(crate) fn new(location: EntityLocation, world: &'a mut World) -> Self {
        Self {
            key: location,
            world: world,
        }
    }

    pub fn entity_type(&self) -> &EntityType {
        &self.world.entity_types()[self.location.entity_type()]
    }

    pub fn location(&self) -> EntityLocation {
        self.location
    }
}

