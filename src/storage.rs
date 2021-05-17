use crate::entity::{
    Entity,
};
use crate::component::{
    Component,
    ComponentTypeIndex,
};
use std::any::TypeId;
use std::rc::Rc;
use std::collections::HashMap;


struct EntityTypeIndex {
    id: usize,
}

/// The index of a component.
struct ComponentIndex {
    id: usize,
}

/// The components in an entity, along with the constructors to contruct another instance of 
/// and entity kind.
struct EntityType {
    name: Option<String>,
    components: Vec<ComponentTypeIndex>,
    constructors: Vec<fn() -> Box<dyn UnsafeComponentStorage>>,
}

/// A collection of entities with the same layout. We create a new map every time
/// a new entity layout is registered.
struct EntityTypeMap {
    index: EntityTypeIndex,
    entities: Vec<Entity>,
    layout: Rc<EntityType>,
}


/// The location of an entity and one of its components.
struct EntityLocation {
    type_id: EntityTypeIndex, component_id: ComponentIndex
}

impl EntityLocation {
    #[inline]
    fn entity_type(self) -> EntityTypeIndex {
        self.type_id
    }

    #[inline]
    fn component(self) -> ComponentIndex {
        self.component_id
    }
}

/// A map of active entities to the locations of their components.
struct EntityLocationMap {
    locations: HashMap<Entity, EntityTypeIndex>,
}


struct ComponentSlice<'a, T> {
    slice: &'a [T],
}

impl<'a, T> ComponentSlice<'a, T>{
    fn new(slice: &'a [T]) -> ComponentSlice<'a, T> {
        ComponentSlice {
            slice: slice,
        }
    }
}

struct ComponentSliceMut<'a, T> {
    slice: &'a mut [T],
}

impl<'a, T> ComponentSliceMut<'a, T>{
    fn new(slice: &'a mut [T]) -> ComponentSliceMut<'a, T> {
        ComponentSliceMut {
            slice: slice,
        }
    }
}


trait UnsafeComponentStorage: Send + Sync {
    fn swap_remove(&mut self, entity_type: EntityTypeIndex, index: ComponentIndex);

    fn get(&self, entity_type: EntityTypeIndex) -> Option<(*const u8, usize)>;

    unsafe fn get_mut(&mut self, entity_type: EntityTypeIndex) -> Option<(*mut u8, usize)>;

    unsafe fn extended_memcopy(&mut self, entity_type: EntityTypeIndex, ptr: *const u8, len: usize) -> usize;
}

trait ComponentStorage<'a, T: Component>: UnsafeComponentStorage + Default {
    type Iter: Iterator<Item = ComponentSlice<'a, T>>;
    type IterMut: Iterator<Item = ComponentSliceMut<'a, T>>;


    unsafe fn extended_memcopy(&mut self, entity_type: EntityTypeIndex, ptr: *const T, len: usize) -> usize; 

    fn get(&self, entity: Entity) -> Option<&'a T>;

    fn get_mut(&mut self, entity: Entity) -> Option<&'a mut T>;

    fn by_entity_type(&self, entity_type: EntityTypeIndex) -> Option<ComponentSlice<'a, T>>;

    fn by_entity_type_mut(&mut self, entity_type: EntityTypeIndex) -> Option<ComponentSliceMut<'a, T>>;

    fn iter(&self) -> Self::Iter;

    fn iter_mut(&mut self) -> Self::IterMut;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

trait StoreComponentsIn<T> where T: Component {
    type Storage: for<'a> ComponentStorage<'a, T>;
}

