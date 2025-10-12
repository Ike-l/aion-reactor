use std::any::{Any, TypeId};

use crate::memory::access_checked_resource_map::resource::raw_resource::RawResource;

pub mod resource_map;
pub mod raw_resource;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ResourceId(TypeId);

impl From<TypeId> for ResourceId {
    fn from(value: TypeId) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
pub struct Resource(pub RawResource<Box<dyn Any>>);

