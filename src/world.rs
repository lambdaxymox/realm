use crate::component::{
    Component,
    ComponentTypeIndex,
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
};
use downcast::{
    Downcast,
};
use std::collections::{
    HashMap,
    HashSet,
};
use std::mem;


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
    ) -> Self {
        let initial_count = entity_type.entities().len();
        Self {
            entity_type_index: entity_type_index,
            entity_type,
            components: components,
            claimed: 0,
            initial_count: initial_count,
        }
    }

    pub fn insert(&mut self, entity: Entity) {
        todo!()
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
}


pub trait ComponentSource {
    fn push_components<'a>(
        &mut self,
        writer: &mut EntityTypeWriter<'a>,
        entities: impl Iterator<Item = Entity>,
    );
}

pub struct SingleEntity<T> {
    data: T,
}

impl<T1, T2> ComponentSource for SingleEntity<(T1, T2)> 
where
    T1: Component,
    T2: Component,
{
    fn push_components<'a>(
        &mut self,
        writer: &mut EntityTypeWriter<'a>,
        entities: impl Iterator<Item = Entity>,
    ) {
        todo!("IMPLEMENT ME!")
    }
}

pub trait IntoComponentSource {
    type Source: ComponentSource;

    fn into(self) -> Self::Source;
}


impl<T1, T2> IntoComponentSource for (T1, T2)
where 
    T1: Component, 
    T2: Component,
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

    pub fn push<Src: IntoComponentSource>(&mut self, components: Src) -> Entity {
        todo!()
    }

    pub fn extend(&mut self, components: impl IntoComponentSource) -> &[Entity] {
        let mut allocation_buffer = mem::take(&mut self.allocation_buffer);
        allocation_buffer.clear();
        self.extend_out(components, &mut allocation_buffer);
        self.allocation_buffer = allocation_buffer;

        &self.allocation_buffer
    }

    pub fn extend_out<Src, Ext>(&mut self, components: Src, out: &mut Ext)
    where
        Src: IntoComponentSource,
        Ext: for<'a> Extend<&'a Entity>,
    {
        // get the components.
        // get the archetype index for the components. Possibly need to generate one if one.
        // construct a writer for the components.
        // write the components to the collection.
        // get the inserted entities from the writer.
        // write the new entities to out.
        // remove any overwritten entities.
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
        let mut entity_type = &mut self.entity_types[entity_type_index];
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
        todo!("IMPLEMENT ME!")
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


