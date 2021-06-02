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
#[derive(Clone, Default, Debug)]
pub struct EntityLayout {
    components: Vec<ComponentTypeIndex>,
    constructors: Vec<fn() -> Box<dyn OpaqueComponentStorage>>,
}

impl EntityLayout {
    pub fn component_types(&self) -> &[ComponentTypeIndex] {
        &self.components
    }

    pub fn get_constructor_unchecked(
        &self, 
        index: ComponentTypeIndex
    ) -> &fn() -> Box<dyn OpaqueComponentStorage>
    {
        let mut idx = 0;
        for (i, type_id) in self.components.iter().enumerate() {
            if type_id == &index {
                idx = i;
                break;
            }
        }

        &self.constructors[idx]
    }

    pub fn constructors(&self) -> &[fn() -> Box<dyn OpaqueComponentStorage>] {
        &self.constructors
    }

    #[inline]
    pub(crate) fn contains_component<T: Component>(&self) -> bool {
        let type_id = ComponentTypeIndex::of::<T>();
        self.components.contains(&type_id)
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
    pub(crate) fn new(index: EntityTypeIndex, layout: EntityLayout) -> Self {
        Self {
            index: index,
            entities: Vec::new(),
            layout: Arc::new(layout),
        }
    }

    pub fn entities(&self) -> &[Entity] {
        &self.entities
    }

    pub(crate) fn swap_remove(&mut self, entity_index: usize) -> Entity {
        let removed = self.entities.swap_remove(entity_index);

        removed
    }

    pub(crate) fn index(&self) -> EntityTypeIndex {
        self.index
    }

    #[inline]
    pub(crate) fn layout(&self) -> &Arc<EntityLayout> {
        &self.layout
    }

    pub fn contains_component<T: Component>(&self) -> bool {
        self.layout().contains_component::<T>()
    }

    pub(crate) fn contains_component_value(&self, index: usize) -> bool {
        index < self.entities.len()
    }
}

impl ops::Index<EntityTypeIndex> for Vec<EntityType> {
    type Output = EntityType;

    fn index(&self, index: EntityTypeIndex) -> &Self::Output {
        &self[index.id]
    }
}

impl ops::IndexMut<EntityTypeIndex> for Vec<EntityType> {
    fn index_mut(&mut self, index: EntityTypeIndex) -> &mut Self::Output {
        &mut self[index.id]
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityTypeIndex {
    id: usize,
}

impl EntityTypeIndex {
    #[inline]
    pub(crate) fn new(id: usize) -> EntityTypeIndex {
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
    pub fn new(id: usize) -> ComponentIndex {
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

    pub(crate) fn insert(
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

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.locations.keys()
    }

    pub fn get(&self, entity: Entity) -> Option<EntityLocation> {
        self.locations.get(&entity).map(|idx| *idx)
    }

    pub fn set(&mut self, entity: Entity, location: EntityLocation) -> Option<EntityLocation> {
        self.locations.insert(entity, location)
    }

    pub fn remove(&mut self, entity: Entity) -> Option<EntityLocation> {
        self.locations.remove(&entity)
    }
}

#[derive(Debug)]
pub struct ComponentView<'a, T> {
    slice: &'a [T],
}

impl<'a, T> ComponentView<'a, T>{
    #[inline]
    pub (crate) fn new(slice: &'a [T]) -> ComponentView<'a, T> {
        ComponentView {
            slice: slice,
        }
    }

    #[inline]
    pub fn into_slice(self) -> &'a [T] {
        self.slice
    }
}

impl<'a, T: Component> Clone for ComponentView<'a, T> {
    fn clone(&self) -> Self {
        ComponentView::new(self.slice)
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

#[derive(Debug)]
pub struct ComponentViewMut<'a, T> {
    slice: &'a mut [T],
}

impl<'a, T> ComponentViewMut<'a, T>{
    #[inline]
    pub(crate) fn new(slice: &'a mut [T]) -> ComponentViewMut<'a, T> {
        ComponentViewMut {
            slice: slice,
        }
    }

    #[inline]
    pub fn into_slice(self) -> &'a mut [T] {
        self.slice
    }
}

impl<'a, T: Component> Clone for ComponentViewMut<'a, T> {
    fn clone(&self) -> Self {
        todo!("IMPLEMENT ME!")
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
    pub(crate) fn of<T: Component>() -> ComponentMetadata {
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

    unsafe fn extend_memcopy_raw(&mut self, entity_type: EntityTypeIndex, ptr: *const u8, count: usize);

    /// Move all the components of a given entity type from one storage to the
    /// other storage.
    fn transfer_entity_type(
        &mut self, 
        src: EntityTypeIndex, 
        dst: EntityTypeIndex, 
        dst_storage: &mut dyn OpaqueComponentStorage,
    );

    /// Move a component from one storage to another storage.
    fn transfer_component(
        &mut self,
        src: EntityTypeIndex,
        src_component: ComponentIndex,
        dst: EntityTypeIndex,
        dst_storage: &mut dyn OpaqueComponentStorage,
    );

    /// Move a component from one entity type to another entity type.
    fn move_component(
        &mut self,
        src: EntityTypeIndex,
        index: ComponentIndex,
        dst: EntityTypeIndex,
    );

    /// Create a new slice for the given Entity type.
    fn insert_entity_type(&mut self, entity_type: EntityTypeIndex);
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

