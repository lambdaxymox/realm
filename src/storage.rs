use crate::entity::{
    Entity,
};
use crate::component::{
    Component,
    ComponentTypeIndex,
};
use std::rc::{
    Rc,
};
use std::collections::{
    HashMap,
};
use std::ops;


/// The components in an entity, along with the constructors to contruct another instance of 
/// and entity kind.
pub struct EntityType {
    components: Vec<ComponentTypeIndex>,
    constructors: Vec<fn() -> Box<dyn UnsafeComponentStorage>>,
}

/// A collection of entities with the same layout. We create a new map every time
/// a new entity layout is registered.
pub struct EntityTypeMap {
    index: EntityTypeIndex,
    entities: Vec<Entity>,
    layout: Rc<EntityType>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityTypeIndex {
    id: usize,
}

impl EntityTypeIndex {
    #[inline]
    fn new(id: usize) -> EntityTypeIndex {
        EntityTypeIndex {
            id: id,
        }
    }

    #[inline]
    fn id(self) -> usize {
        self.id
    }
}

/// The index of a component.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComponentIndex {
    id: usize,
}

impl ComponentIndex {
    #[inline]
    fn new(id: usize) -> ComponentIndex {
        ComponentIndex {
            id: id,
        }
    }

    #[inline]
    fn id(self) -> usize {
        self.id
    }
}

/// The location of an entity and one of its components.
pub struct EntityLocation {
    type_id: EntityTypeIndex, 
    component_id: ComponentIndex,
}

impl EntityLocation {
    #[inline]
    pub fn entity_type(self) -> EntityTypeIndex {
        self.type_id
    }

    #[inline]
    pub fn component(self) -> ComponentIndex {
        self.component_id
    }
}

/// A map of active entities to the locations of their components.
#[derive(Clone, Debug)]
pub struct EntityLocationMap {
    locations: HashMap<Entity, EntityTypeIndex>,
}

impl EntityLocationMap {
    pub(crate) fn new() -> EntityLocationMap {
        EntityLocationMap {
            locations: HashMap::new(),
        }
    }

    fn insert(&mut self, entity: Entity, entity_type: EntityTypeIndex) -> Option<EntityTypeIndex> {
        self.locations.insert(entity, entity_type)
    }

    pub(crate) fn len(&self) -> usize {
        self.locations.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.locations.is_empty()
    }

    pub(crate) fn contains(&self, entity: Entity) -> bool {
        self.locations.contains_key(&entity)
    }

    fn get(&self, entity: Entity) -> Option<EntityTypeIndex> {
        self.locations.get(&entity).map(|idx| *idx)
    }

    fn remove(&mut self, entity: Entity) -> Option<EntityTypeIndex> {
        self.locations.remove(&entity)
    }
}


pub struct ComponentSlice<'a, T> {
    slice: &'a [T],
}

impl<'a, T> ComponentSlice<'a, T>{
    #[inline]
    fn new(slice: &'a [T]) -> ComponentSlice<'a, T> {
        ComponentSlice {
            slice: slice,
        }
    }
}

impl<'a, T: Component> ops::Deref for ComponentSlice<'a, T> {
    type Target = [T];

    fn deref(&self) -> &'a Self::Target {
        self.slice
    }
}

impl<'a, T: Component> From<ComponentSlice<'a, T>> for &'a [T] {
    fn from(components: ComponentSlice<'a, T>) -> Self {
        components.slice
    }
}

impl<'a, T> ops::Index<ComponentIndex> for ComponentSlice<'a, T> {
    type Output = T;

    fn index(&self, index: ComponentIndex) -> &Self::Output {
        &self.slice[index.id]
    }
}

pub struct ComponentSliceMut<'a, T> {
    slice: &'a mut [T],
}

impl<'a, T> ComponentSliceMut<'a, T>{
    #[inline]
    fn new(slice: &'a mut [T]) -> ComponentSliceMut<'a, T> {
        ComponentSliceMut {
            slice: slice,
        }
    }
}

impl<'a, T: Component> ops::Deref for ComponentSliceMut<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.slice
    }
}

impl<'a, T> ops::Index<ComponentIndex> for ComponentSliceMut<'a, T> {
    type Output = T;

    fn index(&self, index: ComponentIndex) -> &Self::Output {
        &self.slice[index.id]
    }
}

impl<'a, T> ops::IndexMut<ComponentIndex> for ComponentSliceMut<'a, T> {
    fn index_mut(&mut self, index: ComponentIndex) -> &mut Self::Output {
        &mut self.slice[index.id]
    }
}


pub trait UnsafeComponentStorage: Send + Sync {
    fn swap_remove(&mut self, entity_type: EntityTypeIndex, index: ComponentIndex);

    fn get(&self, entity_type: EntityTypeIndex) -> Option<(*const u8, usize)>;

    unsafe fn get_mut(&mut self, entity_type: EntityTypeIndex) -> Option<(*mut u8, usize)>;

    unsafe fn extend_memcopy(&mut self, entity_type: EntityTypeIndex, ptr: *const u8, len: usize) -> usize;
}

pub trait ComponentStorage<'a, T: Component>: UnsafeComponentStorage + Default {
    type Iter: Iterator<Item = ComponentSlice<'a, T>>;
    type IterMut: Iterator<Item = ComponentSliceMut<'a, T>>;

    fn get(&self, entity_type: EntityTypeIndex) -> Option<ComponentSlice<'a, T>>;

    fn get_mut(&mut self, entity_type: EntityTypeIndex) -> Option<ComponentSliceMut<'a, T>>;

    unsafe fn extend_memcopy(&mut self, entity_type: EntityTypeIndex, ptr: *const T, len: usize) -> usize; 

    fn iter(&self) -> Self::Iter;

    fn iter_mut(&mut self) -> Self::IterMut;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait StoreComponentsIn<T> where T: Component {
    type Storage: for<'a> ComponentStorage<'a, T>;
}

