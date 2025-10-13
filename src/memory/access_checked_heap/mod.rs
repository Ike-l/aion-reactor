use std::sync::Mutex;

use crate::memory::{access_checked_heap::{heap::{heap::Heap, HeapId, HeapObject}, heap_access_map::HeapAccessMap}, access_map::Access, errors::{DeResolveError, ResolveError}, ResourceId};

pub mod heap;
pub mod heap_access_map;



#[derive(Debug, Default)]
pub struct AccessCheckedHeap {
    access_map: Mutex<HeapAccessMap>,
    heap: Heap,
}

impl AccessCheckedHeap {
    pub fn test_resource(&self, heap_id: &HeapId) -> bool {
        self.heap.contains(heap_id)
    }

    pub fn test_access(&self, heap_id: &HeapId, access: &Access) -> bool {
        // self.heap.test_access(heap_id, access)
        let access_map = self.access_map.lock().unwrap();
        access_map.test_access(heap_id, access)
    }

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
        self.access_map.lock().unwrap().deaccess(access, heap_id)
    }

    pub fn get_shared<T: 'static>(&self, heap_id: &HeapId) -> Result<(&T, HeapAccessMap), ResolveError> {
        self.access_map.lock().unwrap().access_shared(heap_id.clone())?;
        
        let mut access_map = HeapAccessMap::default();
        access_map.access_shared(heap_id.clone())?;
        todo!("If it fails needs to de access");
        // Safety:
        // Accesses are tracked
        unsafe {
            Ok((self.heap.get(&heap_id).ok_or(ResolveError::NoResource(ResourceId::from(heap_id.clone())))?, access_map))
        }
    }

    pub fn get_unique<T: 'static>(&self, heap_id: &HeapId) -> Result<(&mut T, HeapAccessMap), ResolveError> {
        self.access_map.lock().unwrap().access_unique(heap_id.clone())?;
        
        let mut access_map = HeapAccessMap::default();
        access_map.access_unique(heap_id.clone())?;
        todo!("If it fails needs to de access");
        // Safety:
        // Accesses are tracked
        unsafe {
            Ok((self.heap.get_mut(&heap_id).ok_or(ResolveError::NoResource(ResourceId::from(heap_id.clone())))?, access_map))
        }
    }
}