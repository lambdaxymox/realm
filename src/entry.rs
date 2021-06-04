use crate::component::{
    Component,
};
use crate::storage::{
    EntityType,
    EntityLocation,
    ComponentStorage,
    StoreComponentsIn,
};
use crate::world::{
    World,
};


pub struct Entry<'a> {
    location: EntityLocation,
    world: &'a mut World,
}

impl<'a> Entry<'a> {
    pub(crate) fn new(location: EntityLocation, world: &'a mut World) -> Self {
        Self {
            location,
            world: world,
        }
    }

    pub fn entity_type(&self) -> &EntityType {
        &self.world.entity_types()[self.location.entity_type().id()]
    }

    pub fn location(&self) -> EntityLocation {
        self.location
    }

    pub fn get_component<T>(&self) -> Result<&T, ()> 
    where
        T: Component + StoreComponentsIn,
    {
        let component = self.location.component();
        let entity_type = self.location.entity_type();
        self.world
            .components()
            .get_view::<T>()
            .and_then(move |storage| storage.get(entity_type))
            .and_then(move |view| view.into_slice().get(component.id()))
            .ok_or_else(|| {

            })
    }
}

