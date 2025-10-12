use std::sync::Mutex;

use crate::memory::{access_checked_heap::{access::{access_map::AccessMap, Access}, heap::{heap::Heap, HeapObject}}, errors::{DeResolveError, ResolveError}, ResourceId};

pub mod heap;
pub mod access;



#[derive(Debug, Default)]
pub struct AccessCheckedHeap {
    access_map: Mutex<AccessMap>,
    heap: Heap,
}

impl AccessCheckedHeap {
    pub fn insert(&self, resource_id: ResourceId, resource: HeapObject) -> Option<HeapObject> {
        let access_map = self.access_map.lock().unwrap();
        if let Some(_) = access_map.access(&resource_id) {
            return None
        }

        // Safety:
        // Accesses are tracked
        unsafe { self.heap.insert(resource_id, resource) }
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