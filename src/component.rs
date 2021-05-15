use crate::entity::{
    Entity,
};
use std::any::TypeId;

struct EntityTypeIndex {
    id: usize,
}

struct ComponentTypeIndex {
    type_id: TypeId,
}


/// The index of a component.
struct ComponentIndex {
    id: usize,
}


/// The location of an entity and one of its components.
struct EntityLocation {
    type_id: EntityTypeIndex,
    component_id: ComponentIndex,
}


struct ComponentSlice<'a, T> {
    slice: &'a [T],
}

struct ComponentSliceMut<'a, T> {
    slice: &'a mut [T],
}

trait ComponentStorage<'a, T: Component> {
    type Iter: Iterator<Item = ComponentSlice<'a, T>>;
    type IterMut: Iterator<Item = ComponentSliceMut<'a, T>>;


    unsafe fn insert(&mut self, entity_type: EntityTypeIndex, component: *const T) -> EntityLocation; 

    fn get(&self, entity: Entity) -> Option<()>;

    fn get_mut(&mut self, entity: Entity) -> Option<()>;

    fn by_entity_type(&self, entity_type: EntityTypeIndex) -> Option<ComponentSlice<'a, T>>;

    fn by_entity_type_mut(&mut self, entity_type: EntityTypeIndex) -> Option<ComponentSliceMut<'a, T>>;

    fn iter(&self) -> Self::Iter;

    fn iter_mut(&mut self) -> Self::IterMut;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}


trait Component: 'static + Sized + Send + Sync {}

impl<T> Component for T where T: 'static + Sized + Send + Sync {}


trait StoreComponentsIn<T> where T: Component {
    type Storage: for<'a> ComponentStorage<'a, T>;
}

