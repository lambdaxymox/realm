use std::any::{
    TypeId,
};
use std::fmt;


pub trait Component: 'static + Sized + Send + Sync {}

impl<T> Component for T where T: 'static + Sized + Send + Sync {}


#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ComponentTypeIndex {
    type_id: TypeId,
}

impl ComponentTypeIndex {
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub fn of<T: Component>(&self) -> ComponentTypeIndex {
        ComponentTypeIndex {
            type_id: TypeId::of::<T>(),
        }
    }
}

impl fmt::Display for ComponentTypeIndex {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self.type_id)
    }
}

