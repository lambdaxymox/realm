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
use std::alloc;
use std::mem;
use std::ops;
use std::ptr;
use std::ptr::{
    NonNull,
};
use std::slice;
use std::slice::{
    Iter,
    IterMut,
};

#[derive(Debug)]
struct RawComponentArray<T> {
    ptr: NonNull<T>,
    capacity: usize,
}

impl<T> RawComponentArray<T> {
    fn with_capacity(capacity: usize) -> Self {
        if mem::size_of::<T>() == 0 {
            Self {
                ptr: NonNull::dangling(),
                capacity: usize::MAX,
            }
        } else if capacity == 0 {
            Self {
                ptr: NonNull::dangling(),
                capacity: 0,
            }
        } else {
            let layout = alloc::Layout::from_size_align(
                mem::size_of::<T>() * capacity, 
                mem::align_of::<T>()
            )
            .unwrap();
            
            let raw_ptr = unsafe {
                alloc::alloc(layout) as *mut T
            };

            Self {
                ptr: NonNull::new(raw_ptr).unwrap(),
                capacity: capacity,
            }
        }
    }

    fn grow(&mut self, new_capacity: usize) {
        debug_assert!(self.capacity < new_capacity);
        unsafe {
            let dst_ptr = if self.capacity == 0 {
                // If the old capacity is zero, we allocated zero space in the old allocation.
                let layout = alloc::Layout::from_size_align(
                    mem::size_of::<T>() * new_capacity,
                    mem::align_of::<T>()
                )
                .unwrap();
                let new_allocation = alloc::alloc(layout);
                
                new_allocation as *mut T
            } else {
                let layout = alloc::Layout::from_size_align(
                    mem::size_of::<T>() * new_capacity, 
                    mem::align_of::<T>()
                )
                .unwrap();

                let new_allocation = alloc::realloc(
                    self.ptr.as_ptr() as *mut u8,
                    layout,
                    mem::size_of::<T>() * new_capacity
                );
                
                new_allocation as *mut T
            };
            if let Some(new_ptr) = NonNull::new(dst_ptr) {
                self.ptr = new_ptr;
                self.capacity = new_capacity;
            } else {
                let layout = alloc::Layout::from_size_align_unchecked(
                    mem::size_of::<T>() * new_capacity, 
                    mem::align_of::<T>()
                );

                alloc::handle_alloc_error(layout)
            }
        }
    }
}

impl<T> Drop for RawComponentArray<T> {
    fn drop(&mut self) {
        if (mem::size_of::<T>() != 0) && (self.capacity > 0) {
            unsafe {
                let layout = alloc::Layout::from_size_align_unchecked(
                    mem::size_of::<T>() * self.capacity,
                    mem::align_of::<T>(),
                );

                alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
        }
    }
}

#[derive(Debug)]
struct ComponentArray<T> {
    inner: RawComponentArray<T>,
    length: usize,
    capacity: usize,
}

impl<T> ComponentArray<T> {
    fn new() -> Self {
        Self {
            inner: RawComponentArray::with_capacity(0),
            length: 0,
            capacity: 0,
        }
    }

    fn swap_remove(&mut self, index: usize) -> T {
        let (ptr, len) = self.as_raw_slice();
        debug_assert!(index < len);
        unsafe {
            let item_ptr = ptr.as_ptr().add(index);
            let last_ptr = ptr.as_ptr().add(len - 1);
            if index < len - 1 {
                // We are removing an item from the middle of the array. If
                // we were removing the last item from the array (i.e. the item at 
                // array position len - 1), there would be no need to swap to keep the 
                // array packed, so we can just remove the last item from the 
                // array and return it.
                ptr::swap(item_ptr, last_ptr);
            }
            let last_value = ptr::read(last_ptr);
            self.length -= 1;

            last_value
        }
    }

    #[inline]
    fn as_raw_slice(&self) -> (NonNull<T>, usize) {
        let raw_ptr = self.inner.ptr.as_ptr();
        let ptr = unsafe {
            NonNull::new_unchecked(raw_ptr)
        };

        (ptr, self.length)
    }

    fn grow(&mut self, new_capacity: usize) {
        self.inner.grow(new_capacity);
    }

    fn reserve(&mut self, additonal: usize) {
        if self.capacity < self.length + additonal {
            self.grow(self.length + additonal);
        }
    }

    unsafe fn extend_memcopy(&mut self, ptr: *const T, count: usize) {
        self.reserve(count);
        let (dst, len) = self.as_raw_slice();
        ptr::copy_nonoverlapping(ptr, dst.as_ptr().add(len), count);
        self.length += count;
    }
}

impl<T> ops::Deref for ComponentArray<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        let (ptr, len) = self.as_raw_slice();
        unsafe {
            slice::from_raw_parts(ptr.as_ptr(), len)
        }
    }
}

impl<T> ops::DerefMut for ComponentArray<T> {
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
    components: Vec<ComponentArray<T>>,
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
        let component_array = ComponentArray::<T>::new();

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
            // If the component array is empty, there is nothing to transfer,
            // so we can just swap the arrays directly.
            mem::swap(
                &mut self.components[src_index], 
                &mut dst_storage.components[dst_index]
            );
        } else {
            let (ptr, len) = self.get_bytes(src).unwrap();
            unsafe {
                dst_storage.extend_memcopy_raw(dst, ptr, len);
            }

            let mut swapped_components = ComponentArray::<T>::new();
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

