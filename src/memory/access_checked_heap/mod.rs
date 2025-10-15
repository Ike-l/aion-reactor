use std::sync::Mutex;

use crate::{memory::{access_checked_heap::{heap::{heap::Heap, HeapId, HeapObject}, heap_access_map::HeapAccessMap}, access_map::Access, errors::{DeResolveError, ResolveError}, memory_domain::MemoryDomain, ResourceId}, system::system_metadata::Source};

pub mod heap;
pub mod heap_access_map;
pub mod reserve_access_map;
pub mod raw_access_map;


#[derive(Debug, Default)]
pub struct AccessCheckedHeap {
    access_map: Mutex<HeapAccessMap>,
    heap: Heap,
}

impl AccessCheckedHeap {
    pub fn ok_resource(&self, heap_id: &HeapId) -> bool {
        self.heap.contains(heap_id)
    }

    pub fn ok_access(&self, testing_heap_id: &HeapId, testing_access: &Access) -> bool {
        self.access_map.lock().unwrap().ok_access(testing_heap_id, testing_access)
    }

    pub fn reserve_accesses(&self, memory_domain: &MemoryDomain, source: Source, access_map: HeapAccessMap) -> bool {
        self.access_map.lock().unwrap().reserve_accesses(memory_domain, source, access_map)
    }

    pub fn insert(&self, heap_id: HeapId, resource: HeapObject) -> Option<HeapObject> {
        let access_map = self.access_map.lock().unwrap();
        if let Some(_) = access_map.get_access(&heap_id) {
            return None
        }

        // Safety:
        // Accesses are tracked
        unsafe { self.heap.insert(heap_id, resource) }
    }

    // pub crate for now since i only want the dropper to use this
    pub(crate) fn deresolve(&self, access: &Access, heap_id: &HeapId) -> Result<(), DeResolveError> {
        println!("Deresolving: {heap_id:?}. Access: {access:?}");
        self.access_map.lock().unwrap().deaccess(access, heap_id)
    }

    pub fn get_cloned<T: 'static + Clone>(&self, heap_id: &HeapId, source: Option<&Source>) -> Result<T, ResolveError> {
        // Safety:
        // Accesses are tracked
        unsafe {
            Ok(self.heap.get::<T>(&heap_id).ok_or(ResolveError::NoResource(ResourceId::from(heap_id.clone())))?.clone())
        }
    }

    pub fn get_shared<T: 'static>(&self, heap_id: &HeapId, source: Option<&Source>) -> Result<&T, ResolveError> {
        self.access_map.lock().unwrap().access_shared(heap_id.clone(), source)?;

        // Safety:
        // Accesses are tracked
        unsafe {
            Ok(self.heap.get(&heap_id).ok_or(ResolveError::NoResource(ResourceId::from(heap_id.clone())))?)
        }
    }

    pub fn get_unique<T: 'static>(&self, heap_id: &HeapId, source: Option<&Source>) -> Result<&mut T, ResolveError> {
        self.access_map.lock().unwrap().access_unique(heap_id.clone(), source)?;
        println!("U Accessed: {heap_id:?}");
        
        // Safety:
        // Accesses are tracked
        unsafe {
            Ok(self.heap.get_mut(&heap_id).ok_or(ResolveError::NoResource(ResourceId::from(heap_id.clone())))?)
        }
    }
}