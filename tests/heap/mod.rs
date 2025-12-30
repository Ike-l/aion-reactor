use aion_reactor::Memory;

use crate::heap::{access_map::AccessMap, heap_error::HeapError, owner::Owner, resource::Resource, resource_id::ResourceId};

pub mod heap_error;
pub mod owner;
pub mod access_map;
pub mod resource_id;
pub mod resource;

#[derive(Default)]
pub struct Heap {

}

impl Memory for Heap {
    type Resource = Resource;

    type ResourceId = ResourceId;

    type AccessMap = AccessMap;

    type Owner = Owner;

    type Error = HeapError;

    fn contains_resource(
        &self, 
        resource_id: &Self::ResourceId
    ) -> bool {
        todo!()
    }

    fn can_access_resource(
        &self, 
        access: &<Self::AccessMap as aion_reactor::prelude::AccessMap>::Access, 
        owner: Option<&Self::Owner>
    ) -> bool {
        todo!()
    }

    fn reserve(
        &self,
        owner: Self::Owner,
        accesses: Self::AccessMap,
    ) -> Option<Self::Error> {
        todo!()
    }

    fn unreserve(
        &self,
        owner: &Self::Owner,
        accesses: Self::AccessMap,
    ) -> bool {
        todo!()
    }

    fn insert<T: 'static>(
        &self,
        resource_id: &Self::ResourceId,
        resource: T,
    ) -> Option<Self::Error> {
        todo!()
    }

    fn remove<T: 'static>(
        &self,
        resource_id: &Self::ResourceId
    ) -> Result<T, Self::Error> {
        todo!()
    }

    fn get<T: 'static>(
        &self,
        resource_id: &Self::ResourceId
    ) -> Result<&T, Self::Error> {
        todo!()
    }

    fn get_mut<T: 'static>(
        &self,
        resource_id: &Self::ResourceId
    ) -> Result<&mut T, Self::Error> {
        todo!()
    }

    fn get_cloned<T: 'static + Clone>(
        &self,
        resource_id: &Self::ResourceId
    ) -> Result<T, Self::Error> {
        todo!()
    }
}