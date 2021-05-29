use crate::component::{
    Component,
};
use crate::entity::{
    Entity,
};
use crate::storage::{
    OpaqueComponentStorage,
    ComponentStorage,
    EntityTypeIndex,
    EntityLocation,
    ComponentView,
    ComponentViewMut,
    ComponentMetadata,
    ComponentIndex,
};
use std::slice::{
    Iter,
    IterMut,
};

type ComponentVec<T> = Vec<T>;


pub struct ComponentIter<'a, T> {
    iter: Iter<'a, ComponentView<'a, T>>,
}

impl<'a, T> Iterator for ComponentIter<'a, T> 
where
    T: Component,
{
    type Item = ComponentView<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().cloned()
    }
}

pub struct ComponentIterMut<'a, T> {
    _marker: std::marker::PhantomData<&'a T>,
}

impl<'a, T> Iterator for ComponentIterMut<'a, T> {
    type Item = ComponentViewMut<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!("IMPLEMENT ME!")
    }
}

pub struct PackedStorage<T: Component> {
    length: usize,
    indices: Vec<usize>,
    allocations: Vec<ComponentVec<T>>,
}

unsafe impl<T: Component> Send for PackedStorage<T> {}
unsafe impl<T: Component> Sync for PackedStorage<T> {}

impl<T> PackedStorage<T>
where
    T: Component
{
    fn index(&self, entity_type_index: EntityTypeIndex) -> usize {
        self.indices[entity_type_index.id()]
    }
}

impl<T> Default for PackedStorage<T> 
where
    T: Component,
{
    fn default() -> Self {
        Self {
            length: 0,
            indices: Vec::new(),
            allocations: Vec::new(),
        }
    }
}

impl<T> OpaqueComponentStorage for PackedStorage<T>
where
    T: Component
{
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::of::<T>()
    }

    fn swap_remove(&mut self, entity_type: EntityTypeIndex, index: ComponentIndex) {
        todo!("IMPLEMENT ME!")
    }

    fn get_bytes(&self, entity_type: EntityTypeIndex) -> Option<(*const u8, usize)> {
        todo!("IMPLEMENT ME!")
    }

    unsafe fn get_bytes_mut(&mut self, entity_type: EntityTypeIndex) -> Option<(*mut u8, usize)> {
        todo!("IMPLEMENT ME!")
    }

    unsafe fn extend_memcopy(&mut self, entity_type: EntityTypeIndex, ptr: *const u8, len_bytes: usize) {
        todo!("IMPLEMENT ME!")
    }

    /// Move all the components of a given entity type from one storage to the
    /// other storage.
    fn transfer_entity_type(
        &mut self, 
        src: EntityTypeIndex, 
        dst: EntityTypeIndex, 
        dst_storage: &mut dyn OpaqueComponentStorage,
    ) {
        todo!("IMPLEMENT ME!")
    }

    /// Move a component from one storage to another storage.
    fn transfer_component(
        &mut self,
        src: EntityTypeIndex,
        dst: EntityTypeIndex,
        dst_storage: &mut dyn OpaqueComponentStorage,
    ) {
        todo!("IMPLEMENT ME!")
    }

    /// Move a component from one entity type to another entity type.
    fn move_component(
        &mut self,
        src: EntityTypeIndex,
        index: ComponentIndex,
        dst: EntityTypeIndex,
    ) {
        todo!("IMPLEMENT ME!")
    }

    /// Create a new slice for the given Entity type.
    fn insert_entity_type(&mut self, entity_type: EntityTypeIndex) {
        todo!("IMPLEMENT ME!")
    }
}

impl<'a, T> ComponentStorage<'a, T> for PackedStorage<T>
where 
    T: Component,
{
    type Iter = ComponentIter<'a, T>;
    type IterMut = ComponentIterMut<'a, T>;

    fn get(&self, entity_type: EntityTypeIndex) -> Option<ComponentView<'a, T>> {
        todo!("IMPLEMENT ME!")
    }

    fn get_mut(&mut self, entity_type: EntityTypeIndex) -> Option<ComponentViewMut<'a, T>> {
        todo!("IMPLEMENT ME!")
    }

    unsafe fn extend_memcopy(&mut self, entity_type: EntityTypeIndex, ptr: *const T, len: usize) {
        todo!("IMPLEMENT ME!")
    }

    fn iter(&self) -> Self::Iter {
        todo!("IMPLEMENT ME!")
    }

    fn iter_mut(&mut self) -> Self::IterMut {
        todo!("IMPLEMENT ME!")
    }

    fn len(&self) -> usize {
        todo!("IMPLEMENT ME!")
    }
}

