use std::sync::Arc;

use crate::{injection::{injection_trait::Injection, AccessDropper}, memory::{access_checked_heap::{access::Access, AccessCheckedHeap,}, errors::{DeResolveError, ResolveError}, resource_id::Resource, ResourceId}};

// Should be no public way of creating one of these to enforce dropping behaviour by injection types
#[derive(Debug)]
pub struct MemoryDomain {
    heap: AccessCheckedHeap
}

impl MemoryDomain {
    #[allow(dead_code)]
    #[cfg(test)]
    pub(crate) fn new() -> Self {
        Self {
            heap: AccessCheckedHeap::default()
        }
    }

    pub fn insert(&self, resource_id: ResourceId, resource: Resource) -> Option<Resource> {
        match resource {
            Resource::Heap(obj) => Some(Resource::Heap(self.heap.insert(resource_id, obj)?))
        }
    }

    pub fn resolve<T: Injection>(self: &Arc<Self>, resource_id: Option<ResourceId>) -> Result<T::Item<'_>, ResolveError> {
        let r = T::retrieve(&self, resource_id);
        if let Ok(r) = &r {
            // make sure no panics so there MUST be a dropper
            std::hint::black_box(r.access_dropper());
        }

        r
    }

    // pub crate for now since i only want the dropper to use this
    pub(crate) fn deresolve(&self, access: Access, resource: &ResourceId) -> Result<(), DeResolveError> {
        self.heap.deresolve(access, resource)
    }

    pub fn get_shared<T: 'static>(&self, resource_id: ResourceId) -> Result<&T, ResolveError> {
        self.heap.get_shared(resource_id)
    }

    pub fn get_unique<T: 'static>(&self, resource_id: ResourceId) -> Result<&mut T, ResolveError> {
        self.heap.get_unique(resource_id)
    }
}