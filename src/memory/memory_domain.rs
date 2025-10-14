use std::sync::Arc;

use crate::{injection::{injection_trait::Injection, AccessDropper}, memory::{access_checked_heap::AccessCheckedHeap, access_map::{Access, AccessMap}, errors::{DeResolveError, ResolveError}, resource_id::Resource, ResourceId}};

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

    pub fn ok_resource(&self, resource_id: &ResourceId) -> bool {
        match resource_id {
            ResourceId::Heap(heap_id) => self.heap.ok_resource(&heap_id)
        }
    }

    pub fn ok_access(&self, resource_id: &ResourceId, access: &Access) -> bool {
        match resource_id {
            ResourceId::Heap(heap_id) => self.heap.ok_access(&heap_id, access)
        }
    }

    pub fn reserve_accesses(&self, source: ResourceId, access_map: AccessMap) -> bool {
        match access_map {
            AccessMap::Heap(access_map) => self.heap.reserve_accesses(&self, source, access_map)
        }
    }

    pub fn insert(&self, resource_id: ResourceId, resource: Resource) -> Option<Resource> {
        match (resource, resource_id) {
            (Resource::Heap(obj), ResourceId::Heap(id)) => Some(Resource::Heap(self.heap.insert(id, obj)?))
        }
    }

    pub fn resolve<T: Injection>(self: &Arc<Self>, resource_id: Option<&ResourceId>, source: Option<&ResourceId>) -> Result<T::Item<'_>, ResolveError> {
        let r = T::retrieve(&self, resource_id, source);
        if let Ok(r) = &r {
            // make sure no panics so there MUST be a dropper
            std::hint::black_box(r.access_dropper());
        }

        r
    }

    // pub crate for now since i only want the dropper to use this
    pub(crate) fn deresolve(&self, access: &Access, resource_id: &ResourceId) -> Result<(), DeResolveError> {
        match resource_id {
            ResourceId::Heap(id) => self.heap.deresolve(access, id)
        }
    }

    pub fn get_shared<T: 'static>(&self, resource_id: &ResourceId, source: Option<&ResourceId>) -> Result<&T, ResolveError> {
        match resource_id {
            ResourceId::Heap(id) => self.heap.get_shared(id, source)
        }
    }

    pub fn get_unique<T: 'static>(&self, resource_id: &ResourceId, source: Option<&ResourceId>) -> Result<&mut T, ResolveError> {
        match resource_id {
            ResourceId::Heap(id) => self.heap.get_unique(id, source)
        }
    }
}