use std::sync::Mutex;

use crate::memory::{access_checked_heap::{access::{access_map::HeapAccessMap, Access}, heap::{heap::Heap, HeapId, HeapObject}}, errors::{DeResolveError, ResolveError},ResourceId};

pub mod heap;
pub mod access;



#[derive(Debug, Default)]
pub struct AccessCheckedHeap {
    access_map: Mutex<HeapAccessMap>,
    heap: Heap,
}

impl AccessCheckedHeap {
    pub fn insert(&self, heap_id: HeapId, resource: HeapObject) -> Option<HeapObject> {
        let access_map = self.access_map.lock().unwrap();
        if let Some(_) = access_map.access(&heap_id) {
            return None
        }

        // Safety:
        // Accesses are tracked
        unsafe { self.heap.insert(heap_id, resource) }
    }

    // pub crate for now since i only want the dropper to use this
    pub(crate) fn deresolve(&self, access: Access, heap_id: &HeapId) -> Result<(), DeResolveError> {
        todo!()
        // Ok(())
    }

    pub fn get_shared<T: 'static>(&self, heap_id: &HeapId) -> Result<&T, ResolveError> {
        self.access_map.lock().unwrap().access_shared(heap_id.clone())?;
        // Safety:
        // Accesses are tracked
        unsafe { self.heap.get(&heap_id).ok_or(ResolveError::NoResource(ResourceId::from(heap_id.clone()))) }
    }

    pub fn get_unique<T: 'static>(&self, heap_id: &HeapId) -> Result<&mut T, ResolveError> {
        self.access_map.lock().unwrap().access_unique(heap_id.clone())?;
        // Safety:
        // Accesses are tracked
        unsafe { self.heap.get_mut(&heap_id).ok_or(ResolveError::NoResource(ResourceId::from(heap_id.clone()))) }
    }
}