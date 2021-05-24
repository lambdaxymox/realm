use crate::entity::{
    Entity,
};
use crate::component::{
    Component,
    ComponentTypeIndex,
};
use downcast::{
    Downcast,
    impl_downcast,
};
use std::sync::{
    Arc,
};
use std::collections::{
    HashMap,
};
use std::mem;
use std::ops;
use std::ptr;


/// The components in an entity, along with the constructors to contruct another instance of 
/// and entity kind.
#[derive(Debug)]
pub struct EntityLayout {
    components: Vec<ComponentTypeIndex>,
    constructors: Vec<fn() -> Box<dyn OpaqueComponentStorage>>,
}

impl EntityLayout {
    pub fn components(&self) -> &[ComponentTypeIndex] {
        &self.components
    }
}

/// A collection of entities with the same layout. We create a new map every time
/// a new entity layout is registered.
#[derive(Debug)]
pub struct EntityType {
    index: EntityTypeIndex,
    entities: Vec<Entity>,
    layout: Arc<EntityLayout>,
}

impl EntityType {
    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }
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
    pub(crate) fn id(self) -> usize {
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
    pub fn id(self) -> usize {
        self.id
    }
}

/// The location of an entity and its components.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
    locations: HashMap<Entity, EntityLocation>,
}

impl EntityLocationMap {
    pub(crate) fn new() -> EntityLocationMap {
        EntityLocationMap {
            locations: HashMap::new(),
        }
    }

    fn insert(
        &mut self, 
        entities: &[Entity],
        entity_type: EntityTypeIndex, 
        base: ComponentIndex
    ) -> Option<EntityLocation>
    {
        todo!("IMPLEMENT ME!")
    }

    pub fn len(&self) -> usize {
        self.locations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.locations.is_empty()
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.locations.contains_key(&entity)
    }

    pub fn get(&self, entity: Entity) -> Option<EntityLocation> {
        self.locations.get(&entity).map(|idx| *idx)
    }

    pub fn remove(&mut self, entity: Entity) -> Option<EntityLocation> {
        self.locations.remove(&entity)
    }
}

pub struct ComponentView<'a, T> {
    slice: &'a [T],
}

impl<'a, T> ComponentView<'a, T>{
    #[inline]
    fn new(slice: &'a [T]) -> ComponentView<'a, T> {
        ComponentView {
            slice: slice,
        }
    }

    #[inline]
    pub fn into_slice(self) -> &'a [T] {
        self.slice
    }
}

impl<'a, T: Component> ops::Deref for ComponentView<'a, T> {
    type Target = [T];

    fn deref(&self) -> &'a Self::Target {
        self.slice
    }
}

impl<'a, T: Component> From<ComponentView<'a, T>> for &'a [T] {
    fn from(components: ComponentView<'a, T>) -> Self {
        components.slice
    }
}

impl<'a, T> ops::Index<ComponentIndex> for ComponentView<'a, T> {
    type Output = T;

    fn index(&self, index: ComponentIndex) -> &Self::Output {
        &self.slice[index.id]
    }
}

pub struct ComponentViewMut<'a, T> {
    slice: &'a mut [T],
}

impl<'a, T> ComponentViewMut<'a, T>{
    #[inline]
    fn new(slice: &'a mut [T]) -> ComponentViewMut<'a, T> {
        ComponentViewMut {
            slice: slice,
        }
    }

    #[inline]
    pub fn into_slice(self) -> &'a mut [T] {
        self.slice
    }
}

impl<'a, T: Component> ops::Deref for ComponentViewMut<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.slice
    }
}

impl<'a, T> ops::Index<ComponentIndex> for ComponentViewMut<'a, T> {
    type Output = T;

    fn index(&self, index: ComponentIndex) -> &Self::Output {
        &self.slice[index.id]
    }
}

impl<'a, T> ops::IndexMut<ComponentIndex> for ComponentViewMut<'a, T> {
    fn index_mut(&mut self, index: ComponentIndex) -> &mut Self::Output {
        &mut self.slice[index.id]
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct ComponentMetadata {
    size: usize,
    alignment: usize,
    drop_fn: Option<fn(*mut u8)>,
}

impl ComponentMetadata {
    fn of<T: Component>() -> ComponentMetadata {
        let drop_fn: Option<fn(*mut u8)> = if mem::needs_drop::<T>() {
            Some(|ptr| unsafe {
                ptr::drop_in_place(ptr as *mut T)
            })
        } else {
            None
        };

        ComponentMetadata {
            size: mem::size_of::<T>(),
            alignment: mem::align_of::<T>(),
            drop_fn: drop_fn,
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn alignment(&self) -> usize {
        self.alignment
    }

    pub fn padding(&self) -> usize {
        self.alignment - self.size
    }

    pub unsafe fn drop(&self, value: *mut u8) {
        if let Some(drop_fn) = self.drop_fn {
            drop_fn(value)
        }
    }
}

pub trait OpaqueComponentStorage: Downcast + Send + Sync {
    fn metadata(&self) -> ComponentMetadata;

    fn swap_remove(&mut self, entity_type: EntityTypeIndex, index: ComponentIndex);

    fn get_bytes(&self, entity_type: EntityTypeIndex) -> Option<(*const u8, usize)>;

    unsafe fn get_bytes_mut(&mut self, entity_type: EntityTypeIndex) -> Option<(*mut u8, usize)>;

    unsafe fn extend_memcopy(&mut self, entity_type: EntityTypeIndex, ptr: *const u8, len: usize);
}

impl_downcast!(OpaqueComponentStorage);


pub trait ComponentStorage<'a, T: Component>: OpaqueComponentStorage + Default {
    type Iter: Iterator<Item = ComponentView<'a, T>>;
    type IterMut: Iterator<Item = ComponentViewMut<'a, T>>;

    fn get(&self, entity_type: EntityTypeIndex) -> Option<ComponentView<'a, T>>;

    fn get_mut(&mut self, entity_type: EntityTypeIndex) -> Option<ComponentViewMut<'a, T>>;

    unsafe fn extend_memcopy(&mut self, entity_type: EntityTypeIndex, ptr: *const T, len: usize);

    fn iter(&self) -> Self::Iter;

    fn iter_mut(&mut self) -> Self::IterMut;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait StoreComponentsIn where Self: Component {
    type Storage: for<'a> ComponentStorage<'a, Self>;
}

