use std::sync::{Arc, Mutex};

use crate::{injection::{injection_trait::Injection, AccessDropper}, memory::{access_checked_resource_map::{access::{access_map::AccessMap, Access}, heap::{heap::Heap, HeapObject}}, ResourceId}};

pub mod heap;
pub mod access;

#[derive(Debug)]
pub enum ResolveError {
    ConflictingAccess(ResourceId),
    InvalidProgramId,
    NoResource(ResourceId),
}

#[derive(Debug)]
pub enum DeResolveError {

}

// Should be no public way of creating one of these to enforce dropping behaviour by injection types
#[derive(Debug)]
pub struct AccessCheckedHeap {
    access_map: Mutex<AccessMap>,
    heap: Heap,
}

impl AccessCheckedHeap {
    #[allow(dead_code)]
    #[cfg(test)]
    pub(crate) fn new() -> Self {
        Self {
            access_map: Mutex::new(AccessMap::default()),
            heap: Heap::default()
        }
    }

    pub fn insert(&self, resource_id: ResourceId, resource: HeapObject) -> Option<HeapObject> {
        let access_map = self.access_map.lock().unwrap();
        if let Some(_) = access_map.access(&resource_id) {
            return None
        }

        // Safety:
        // Accesses are tracked
        unsafe { self.heap.insert(resource_id, resource) }
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
        todo!()
    }

    pub fn get_shared<T: 'static>(&self, resource_id: ResourceId) -> Result<&T, ResolveError> {
        self.access_map.lock().unwrap().access_shared(resource_id.clone())?;
        // Safety:
        // Accesses are tracked
        unsafe { self.heap.get(&resource_id).ok_or(ResolveError::NoResource(resource_id)) }
    }

    pub fn get_unique<T: 'static>(&self, resource_id: ResourceId) -> Result<&mut T, ResolveError> {
        self.access_map.lock().unwrap().access_unique(resource_id.clone())?;
        // Safety:
        // Accesses are tracked
        unsafe { self.heap.get_mut(&resource_id).ok_or(ResolveError::NoResource(resource_id)) }
    }
}