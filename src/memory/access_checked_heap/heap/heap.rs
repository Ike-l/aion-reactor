use crate::memory::{access_checked_heap::heap::{raw_heap::RawHeap, HeapObject}, ResourceId};

#[derive(Debug, Default)]
pub struct Heap {
    lock: parking_lot::RwLock<()>,
    raw_heap: RawHeap
}

impl Heap {
    /// Safety:
    /// Ensure no concurrent mutable accesses
    pub unsafe fn get<T: 'static>(&self, resource_id: &ResourceId) -> Option<&T> {
        let guard = self.lock.read();
        unsafe { self.raw_heap.get(resource_id, guard) }
    }

    /// Safety:
    /// Ensure no concurrent accesses
    pub unsafe fn get_mut<T: 'static>(&self, resource_id: &ResourceId) -> Option<&mut T> {
        let guard = self.lock.read();
        unsafe { self.raw_heap.get_mut(resource_id, guard) }
    }

    /// Safety:
    /// Ensure no concurrent accesses
    pub unsafe fn insert(&self, resource_id: ResourceId, resource: HeapObject) -> Option<HeapObject> {
        let guard = self.lock.write();
        unsafe { self.raw_heap.insert(resource_id, resource, guard) }
    }
}