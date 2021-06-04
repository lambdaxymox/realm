use crate::component::{
    Component,
    ComponentTypeIndex,
};
use crate::compactable::{
    CompactableStorage,
};
use crate::entity::{
    Entity,
    EntityAllocator,
};
use crate::storage::{
    OpaqueComponentStorage,
    EntityLocationMap,
    EntityType,
    EntityLayout,
    EntityTypeIndex,
    EntityLocation,
    StoreComponentsIn,
    ComponentStorage,
    ComponentIndex,
};
use downcast::{
    Downcast,
};
use std::collections::{
    HashMap,
    HashSet,
};
use std::mem;
use std::ops::{
    DerefMut,
};


/// where the components live in a world.
pub struct ComponentMap {
    data: HashMap<ComponentTypeIndex, Box<dyn OpaqueComponentStorage>>,
}

impl ComponentMap {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn get_or_insert_with<F>(
        &mut self,
        index: ComponentTypeIndex,
        mut constructor: F,
    ) -> &mut dyn OpaqueComponentStorage
    where
        F: FnMut() -> Box<dyn OpaqueComponentStorage>,
    {
        let new_storage = self.data
            .entry(index)
            .or_insert_with(constructor);
        
        new_storage.deref_mut()
    }

    fn get(&self, component_type: ComponentTypeIndex) -> Option<&dyn OpaqueComponentStorage> {
        self.data.get(&component_type).map(|cell| cell.as_ref())
    }

    fn get_mut(&mut self, component_type: ComponentTypeIndex) -> Option<&mut dyn OpaqueComponentStorage> {
        self.data
            .get_mut(&component_type)
            .map(|cell| cell.as_mut())
    }

    pub fn get_view<T: Component + StoreComponentsIn>(&self) -> Option<&T::Storage> {
        let component_type = ComponentTypeIndex::of::<T>();
        self.get(component_type)
            .and_then(|storage| storage.downcast_ref())
    }

    pub fn get_view_mut<T: Component + StoreComponentsIn>(&mut self) -> Option<&mut T::Storage> {
        let component_type = ComponentTypeIndex::of::<T>();
        self.get_mut(component_type)
            .and_then(|storage| storage.downcast_mut())
    }

    pub fn contains_component<T: Component + StoreComponentsIn>(&self) -> bool {
        let component_type = ComponentTypeIndex::of::<T>();
        self.data.contains_key(&component_type)
    }

    pub(crate) fn contains_component_id(&self, index: ComponentTypeIndex) -> bool {
        self.data.contains_key(&index)
    }

    pub fn get_multi_view_mut(&mut self) -> MultiViewMut {
        MultiViewMut::new(self)
    }
}

pub struct Entry<'a> {
    location: EntityLocation,
    world: &'a mut World,
}

impl<'a> Entry<'a> {
    fn new(location: EntityLocation, world: &'a mut World) -> Self {
        Self {
            location: location,
            world: world,
        }
    }

    pub fn entity_type(&self) -> &EntityType {
        &self.world.entity_types()[self.location.entity_type().id()]
    }

    pub fn location(&self) -> EntityLocation {
        self.location
    }

    pub fn get_component<T: Component + StoreComponentsIn>(&self) -> Result<&T, ()> {
        let entity_type = self.location.entity_type();
        let component = self.location.component();
        self.world
            .components()
            .get_view::<T>()
            .and_then(move |storage| storage.get(entity_type))
            .and_then(move |view| view.into_slice().get(component.id()))
            .ok_or_else(|| {})
    }

    pub fn get_component_mut<T: Component + StoreComponentsIn>(&mut self) -> Result<&mut T, ()> {
        let entity_type = self.location.entity_type();
        let component = self.location.component();
        self.world
            .components_mut()
            .get_view_mut::<T>()
            .and_then(move |storage| storage.get_mut(entity_type))
            .and_then(move |view| view.into_slice().get_mut(component.id()))
            .ok_or_else(|| {})
    }

    pub fn has_component<T: Component + StoreComponentsIn>(&self) -> bool {
        let entity_type_index = self.location.entity_type();
        let entity_type = &self.world.entity_types[entity_type_index];
        
        entity_type.contains_component::<T>()
    }
}


pub struct MultiViewMut<'a> {
    components: &'a mut ComponentMap,
    claimed: HashSet<ComponentTypeIndex>,    
}

impl<'a> MultiViewMut<'a> {
    fn new(components: &'a mut ComponentMap) -> Self {
        Self {
            components: components,
            claimed: HashSet::default(),
        }
    }

    pub unsafe fn claim<T: Component + StoreComponentsIn>(&mut self) -> Option<&'a mut T::Storage> {
        let type_id = ComponentTypeIndex::of::<T>();
        self.claimed.insert(type_id);

        self.components
            .get_view_mut::<T>()
            .map(|storage| {
                mem::transmute::<&mut T::Storage, &'a mut T::Storage>(storage)
            })
    }
}


pub struct EntityTypeWriter<'a> {
    entity_type_index: EntityTypeIndex,
    entity_type: &'a mut EntityType,
    components: MultiViewMut<'a>,
    claimed: u128,
    initial_count: usize,
}

impl<'a> EntityTypeWriter<'a> {
    pub fn new(
        entity_type_index: EntityTypeIndex,
        entity_type: &'a mut EntityType,
        components: MultiViewMut<'a>,
    ) -> Self 
    {
        let initial_count = entity_type.entities().len();
        Self {
            entity_type_index: entity_type_index,
            entity_type,
            components: components,
            claimed: 0,
            initial_count: initial_count,
        }
    }

    /// Push an entity to the entity type collection.
    pub fn push(&mut self, entity: Entity) {
        self.entity_type.push(entity);
    }

    pub fn claim_components<T: Component + StoreComponentsIn>(&mut self) -> ComponentWriter<'a, T> {
        let component_type_id = ComponentTypeIndex::of::<T>();
        let components = unsafe {
            self.components.claim::<T>().unwrap()
        };
        
        ComponentWriter {
            components: components,
            entity_type: self.entity_type_index,
        }
    }

    pub fn entity_type(&self) -> &EntityType {
        &self.entity_type
    }

    pub fn inserted(&self) -> (ComponentIndex, &[Entity]) {
        let start = self.initial_count;
        let index = ComponentIndex::new(start);
        let slice = &self.entity_type.entities()[start..];
        
        (index, slice)
    }
}

pub trait LayoutFilter {
    fn matches_layout(&self, components: &[ComponentTypeIndex]) -> bool;
}

pub trait EntityTypeSource {
    type Filter: LayoutFilter;

    fn filter(&self) -> Self::Filter;

    fn layout(&mut self) -> EntityLayout;
}

pub trait ComponentSource: EntityTypeSource {
    fn push_components<'a>(
        &mut self,
        writer: &mut EntityTypeWriter<'a>,
        entities: impl Iterator<Item = Entity>,
    );
}

impl<T> LayoutFilter for Option<T> where T: LayoutFilter {
    fn matches_layout(&self, components: &[ComponentTypeIndex]) -> bool {
        match self {
            Some(filter) => {
                filter.matches_layout(components)
            }
            None => false,
        }
    }
}

impl<T> EntityTypeSource for Option<T> where T: EntityTypeSource {
    type Filter = Option<T::Filter>;

    fn filter(&self) -> Self::Filter {
        match self {
            Some(provider) => Some(provider.filter()),
            None => None,
        }
    }

    fn layout(&mut self) -> EntityLayout {
        match self {
            Some(provider) => provider.layout(),
            None => EntityLayout::default()
        }
    }
}

impl<T> ComponentSource for Option<T> where T: ComponentSource {
    fn push_components<'a>(
        &mut self,
        writer: &mut EntityTypeWriter<'a>,
        entities: impl Iterator<Item = Entity>,
    ) {
        match self {
            Some(provider) => {
                <T as ComponentSource>::push_components(provider, writer, entities)
            }
            None => {}
        }
    }
}

impl<T> IntoComponentSource for Option<T> where T: IntoComponentSource {
    type Source = Option<T::Source>;
    
    fn into(self) -> Self::Source {
        match self {
            Some(provider) => Some(provider.into()),
            None => None,
        }
    }
}


pub struct SingleEntity<T> {
    data: T,
}

use std::marker::PhantomData;

pub struct PairFilter<T1, T2> {
    _marker: PhantomData<(T1, T2)>,
}

unsafe impl<T1, T2> Send for PairFilter<T1, T2> {}
unsafe impl<T1, T2> Sync for PairFilter<T1, T2> {}

impl<T1, T2> LayoutFilter for PairFilter<T1, T2>
where
    T1: Component,
    T2: Component,
{
    fn matches_layout(&self, components: &[ComponentTypeIndex]) -> bool {
        let type_array = [
            ComponentTypeIndex::of::<T1>(), 
            ComponentTypeIndex::of::<T2>()
        ];

        type_array.len() == components.len() 
            && type_array.iter().all(|type_id| components.contains(type_id))
    }
}

impl<T1, T2> EntityTypeSource for SingleEntity<(T1, T2)>
where
    T1: Component + StoreComponentsIn,
    T2: Component + StoreComponentsIn,
{
    type Filter = PairFilter<T1, T2>;

    fn filter(&self) -> Self::Filter {
        PairFilter {
            _marker: PhantomData,
        }
    }

    fn layout(&mut self) -> EntityLayout {
        let mut layout = EntityLayout::new();
        layout.register_component::<T1>();
        layout.register_component::<T2>();

        layout
    }
}

impl<T1, T2> ComponentSource for SingleEntity<(T1, T2)> 
where
    T1: Component + StoreComponentsIn,
    T2: Component + StoreComponentsIn,
{
    fn push_components<'a>(
        &mut self,
        writer: &mut EntityTypeWriter<'a>,
        mut entities: impl Iterator<Item = Entity>,
    ) {
        let entity = entities.next();
        debug_assert!(entity.is_some());
        writer.push(entity.unwrap());
        let mut writer_t1 = writer.claim_components::<T1>();
        let mut writer_t2 = writer.claim_components::<T2>();
        unsafe {
            writer_t1.extend_memcopy(&self.data.0 as *const T1, 1);
            writer_t2.extend_memcopy(&self.data.1 as *const T2, 1);
        }
    }
}

pub trait IntoComponentSource {
    type Source: ComponentSource;

    fn into(self) -> Self::Source;
}


impl<T1, T2> IntoComponentSource for (T1, T2)
where 
    T1: Component + StoreComponentsIn, 
    T2: Component + StoreComponentsIn,
{
    type Source = SingleEntity<(T1, T2)>;

    fn into(self) -> Self::Source {
        SingleEntity {
            data: self,
        }
    }
}

pub struct ComponentWriter<'a, T: Component + StoreComponentsIn> {
    components: &'a mut T::Storage,
    entity_type: EntityTypeIndex,
}

impl<'a, T> ComponentWriter<'a, T>
where
    T: Component + StoreComponentsIn
{
    pub unsafe fn extend_memcopy(&mut self, ptr: *const T, len: usize) {
        <T::Storage as ComponentStorage<'a,_>>::extend_memcopy(
            &mut self.components, 
            self.entity_type,
            ptr, 
            len
        );
    }
}

/// Where all the data is grouped together.
pub struct World {
    entities: EntityLocationMap,
    entity_types: Vec<EntityType>,
    entity_allocator: EntityAllocator,
    components: ComponentMap,
    allocation_buffer: Vec<Entity>,
}

impl World {
    pub fn new() -> World {
        World {
            entities: EntityLocationMap::new(),
            entity_types: Vec::new(),
            entity_allocator: EntityAllocator::new(),
            components: ComponentMap::new(),
            allocation_buffer: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.entities.contains(entity)
    }

    pub fn contains_component<T: Component + StoreComponentsIn>(&self) -> bool {
        self.components.contains_component::<T>()
    }

    pub fn has_component<T: Component + StoreComponentsIn>(&self, entity: Entity) -> bool {
        if let Some(location) = self.entities.get(entity) {
            let entity_type_index = location.entity_type();
            let entity_type = &self.entity_types[entity_type_index];
            entity_type.contains_component::<T>()
        } else {
            false
        }
    }

    fn get_entity_type_for_components<T>(&mut self, components: &mut T) -> EntityTypeIndex 
    where
        T: EntityTypeSource,
    {
        let search_entities = |filter: &T::Filter| -> Option<EntityTypeIndex> {
            for entity_type in self.entity_types.iter() {
                if filter.matches_layout(entity_type.layout().component_types()) {
                    return Some(entity_type.index());
                }
            }

            None
        };

        let index = search_entities(&components.filter());
        let entity_type_index = if let Some(value) = index {
            value
        } else {
            self.insert_entity_type(components.layout())
        };

        entity_type_index
    }

    fn insert_entity_type(&mut self, layout: EntityLayout) -> EntityTypeIndex {
        let entity_type_index = EntityTypeIndex::new(self.entity_types.len());
        self.entity_types.push(EntityType::new(entity_type_index, layout));
        let entity_type = &self.entity_types[self.entity_types.len() - 1];
        let missing_components: Vec<ComponentTypeIndex> = entity_type
            .layout()
            .component_types()
            .iter()
            .filter(|type_id| {
                !self.components.contains_component_id(**type_id)
            })
            .map(|p| *p)
            .collect();

        for missing_component in missing_components.iter() {
            self.components.get_or_insert_with(*missing_component, || { 
                entity_type.layout().get_constructor_unchecked(*missing_component)()
            });
        }

        entity_type_index
    }

    pub fn push<Src>(&mut self, components: Src) -> Entity
    where
        Option<Src>: IntoComponentSource,
    {
        struct Single(Option<Entity>);

        impl<'a> Extend<&'a Entity> for Single {
            fn extend<I: IntoIterator<Item = &'a Entity>>(&mut self, iter: I) {
                debug_assert!(self.0.is_none());
                let mut iter = iter.into_iter();
                self.0 = iter.next().copied();
                debug_assert!(iter.next().is_none());
            }
        }

        let mut src = Single(None);
        self.extend_out(Some(components), &mut src);
        src.0.unwrap()
    }

    pub fn extend(&mut self, components: impl IntoComponentSource) -> &[Entity] {
        let mut allocation_buffer = mem::take(&mut self.allocation_buffer);
        allocation_buffer.clear();
        self.extend_out(components, &mut allocation_buffer);
        self.allocation_buffer = allocation_buffer;

        &self.allocation_buffer
    }

    pub fn extend_out<Src, Ext>(&mut self, component_source: Src, out: &mut Ext)
    where
        Src: IntoComponentSource,
        Ext: for<'a> Extend<&'a Entity>,
    {
        let replaced_entities = {
            let mut components = component_source.into();
            let entity_type_index = self.get_entity_type_for_components(&mut components);
            let entity_type = &mut self.entity_types[entity_type_index];
            let mut writer = EntityTypeWriter::new(
                entity_type_index,
                entity_type,
                self.components.get_multi_view_mut()
            );
            components.push_components(&mut writer, &mut self.entity_allocator);

            let (base, new_entities) = writer.inserted();
            let replaced = self.entities.insert(new_entities, entity_type_index, base);
            out.extend(new_entities.iter());

            replaced
        };

        for location in replaced_entities {
            self.remove_at_location(location);
        }
    }

    pub fn remove(&mut self, entity: Entity) -> bool {
        if let Some(location) = self.entities.remove(entity) {
            self.remove_at_location(location);

            true
        } else {
            false
        }
    }

    fn remove_at_location(&mut self, location: EntityLocation) {
        let component_index = location.component();
        let entity_type_index = location.entity_type();
        let entity_type = &mut self.entity_types[entity_type_index];
        entity_type.swap_remove(component_index.id());
        for type_id in entity_type.layout().component_types() {
            let storage = self.components.get_mut(*type_id).unwrap();
            storage.swap_remove(entity_type_index, component_index);
        }

        if entity_type.contains_component_value(component_index.id()) {
            let swapped = entity_type.entities()[component_index.id()];
            self.entities.set(swapped, location);
        }
    }

    pub fn clear(&mut self) {
        let entities: Vec<Entity> = self.entities.iter().map(|e| *e).collect();
        for entity in entities.iter() {
            self.remove(*entity);
        }
    }

    pub fn components(&self) -> &ComponentMap {
        &self.components
    }

    pub fn components_mut(&mut self) -> &mut ComponentMap {
        &mut self.components
    }

    pub fn entity_types(&self) -> &[EntityType] {
        &self.entity_types
    }
}


impl<T> StoreComponentsIn for T
where
    T: Component,
{
    type Storage = CompactableStorage<T>;
}

