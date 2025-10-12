use std::any::{Any, TypeId};

use crate::{id::Id, memory::access_checked_resource_map::resource::raw_resource::RawResource};

pub mod resource_map;
pub mod raw_resource;
pub mod raw_resource_map;
pub mod inner_resource_map;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ResourceId {
    RawTypeId(TypeId),
    Id(Id)
}

impl From<TypeId> for ResourceId {
    fn from(value: TypeId) -> Self {
        Self::RawTypeId(value)
    }
}

impl From<Id> for ResourceId {
    fn from(value: Id) -> Self {
        Self::Id(value)
    }
}

#[derive(Debug)]
pub struct Resource(pub RawResource<Box<dyn Any>>);

