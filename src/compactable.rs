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
use std::mem;
use std::ops::{
    Deref,
    DerefMut,
};
use std::ptr::{
    NonNull,
};
use std::slice;
use std::slice::{
    Iter,
    IterMut,
};

#[derive(Clone, Debug)]
struct ComponentVec<T> {
    data: Vec<T>,
}

impl<T> ComponentVec<T> {
    fn new() -> Self {
        Self {
            data: Vec::new(),
        }
    }

    #[inline]
    fn swap_remove(&mut self, index: usize) -> T {
        self.data.swap_remove(index)
    }

    #[inline]
    fn as_raw_slice(&self) -> (NonNull<T>, usize) {
        todo!("IMPLEMENT ME!");
        let raw_ptr = self.data.as_mut_ptr();
        let len = self.data.len();
        let ptr = unsafe {
            NonNull::new_unchecked(raw_ptr)
        };

        (ptr, len)
    }

    unsafe fn extend_memcopy(&mut self, ptr: *const T, count: usize) {
        todo!("IMPLMENT ME!")
    }
}

impl<T> Deref for ComponentVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        let (ptr, len) = self.as_raw_slice();
        unsafe {
            slice::from_raw_parts(ptr.as_ptr(), len)
        }
    }
}

impl<T> DerefMut for ComponentVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let (ptr, len) = self.as_raw_slice();
        unsafe {
            slice::from_raw_parts_mut(ptr.as_ptr(), len)
        }
    }
}


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

pub struct CompactableStorage<T: Component> {
    length: usize,
    indices: Vec<usize>,
    views: Vec<(NonNull<T>, usize)>,
    components: Vec<ComponentVec<T>>,
}

unsafe impl<T: Component> Send for CompactableStorage<T> {}
unsafe impl<T: Component> Sync for CompactableStorage<T> {}

impl<T> CompactableStorage<T>
where
    T: Component
{
    fn swap_remove_internal(
        &mut self, 
        entity_type: EntityTypeIndex, 
        index: ComponentIndex
    ) -> T
    {
        let view_index = self.indices[entity_type.id()];
        let allocation = &mut self.components[view_index];
        let component = allocation.swap_remove(index.id());
        self.update_view(view_index);
        self.length -= 1;

        component
    }

    fn index(&self, entity_type_index: EntityTypeIndex) -> usize {
        self.indices[entity_type_index.id()]
    }

    fn update_view(&mut self, view_index: usize) {
        self.views[view_index] = self.components[view_index].as_raw_slice();
    }
}

impl<T> Default for CompactableStorage<T> 
where
    T: Component,
{
    fn default() -> Self {
        Self {
            length: 0,
            indices: Vec::new(),
            views: Vec::new(),
            components: Vec::new(),
        }
    }
}

impl<T> OpaqueComponentStorage for CompactableStorage<T>
where
    T: Component
{
    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::of::<T>()
    }

    fn swap_remove(&mut self, entity_type: EntityTypeIndex, index: ComponentIndex) {
        self.swap_remove_internal(entity_type, index);
    }

    fn get_bytes(&self, entity_type: EntityTypeIndex) -> Option<(*const u8, usize)> {
        let view_index = *self.indices.get(entity_type.id())?;
        let (ptr, len_bytes) = self.views.get(view_index)?;

        Some((ptr.as_ptr() as *const u8, *len_bytes))
    }

    unsafe fn get_bytes_mut(&mut self, entity_type: EntityTypeIndex) -> Option<(*mut u8, usize)> {
        let view_index = *self.indices.get(entity_type.id())?;
        let (ptr, len_bytes) = self.views.get(view_index)?;

        Some((ptr.as_ptr() as *mut u8, *len_bytes))
    }

    unsafe fn extend_memcopy_raw(&mut self, entity_type_index: EntityTypeIndex, ptr: *const u8, count: usize) {
        let view_index = self.index(entity_type_index);
        let component = &mut self.components[view_index];
        component.extend_memcopy(ptr as *const T, count);
        self.views[view_index] = component.as_raw_slice();
        self.length += count;
    }

    /// Move a component from one entity type to another entity type.
    fn move_component(
        &mut self,
        src: EntityTypeIndex,
        index: ComponentIndex,
        dst: EntityTypeIndex,
    ) {
        let src_view_index = self.index(src);
        let dst_view_index = self.index(dst);

        let src_components = &mut self.components[src_view_index];
        let value = src_components.swap_remove(index.id());

        let dst_components = &mut self.components[dst_view_index];
        unsafe {
            dst_components.extend_memcopy(&value as *const T, 1);
        }

        self.update_view(src_view_index);
        self.update_view(dst_view_index);

        mem::forget(value);
    }

    /// Move a component from one storage to another storage.
    fn transfer_component(
        &mut self,
        src: EntityTypeIndex,
        src_component: ComponentIndex,
        dst: EntityTypeIndex,
        dst_storage: &mut dyn OpaqueComponentStorage,
    ) {
        let component = self.swap_remove_internal(src, src_component);
        unsafe {
            dst_storage.extend_memcopy_raw(
                dst, 
                &component as *const T as *const u8, 
                1
            );
        }
        mem::forget(component);
    }

    /// Create a new slice for the given Entity type.
    fn insert_entity_type(&mut self, entity_type_index: EntityTypeIndex) {
        let view_index = self.views.len();
        let component_array = ComponentVec::<T>::new();

        self.views.insert(view_index, component_array.as_raw_slice());
        self.components.insert(view_index, component_array);

        if entity_type_index.id() >= self.indices.len() {
            self.indices.resize(entity_type_index.id() + 1, usize::MAX);
        }

        self.indices[entity_type_index.id()] = view_index;
    }

    /// Move all the components of a given entity type from one storage to the
    /// other storage.
    fn transfer_entity_type(
        &mut self,
        src: EntityTypeIndex, 
        dst: EntityTypeIndex, 
        dst_opaque_storage: &mut dyn OpaqueComponentStorage,
    ) {
        let dst_storage = dst_opaque_storage.downcast_mut::<Self>().unwrap();
        let src_index = self.index(src);
        let dst_index = dst_storage.index(dst);

        let entity_count = self.components[src_index].len();
        self.length -= entity_count;
        dst_storage.length += entity_count;

        if dst_storage.components[dst_index].len() == 0 {
            // TODO: Optimize this path.
            let (ptr, len) = self.get_bytes(src).unwrap();
            unsafe {
                dst_storage.extend_memcopy_raw(dst, ptr, len);
            }

            let mut swapped_components = ComponentVec::<T>::new();
            mem::swap(&mut self.components[src_index], &mut swapped_components);
            mem::forget(swapped_components);
        } else {
            let (ptr, len) = self.get_bytes(src).unwrap();
            unsafe {
                dst_storage.extend_memcopy_raw(dst, ptr, len);
            }

            let mut swapped_components = ComponentVec::<T>::new();
            mem::swap(&mut self.components[src_index], &mut swapped_components);
            mem::forget(swapped_components);
        }

        self.update_view(src_index);
        dst_storage.update_view(dst_index);
    }
}

impl<'a, T> ComponentStorage<'a, T> for CompactableStorage<T>
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

