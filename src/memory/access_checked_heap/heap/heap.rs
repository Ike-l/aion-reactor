use crate::memory::access_checked_heap::heap::{raw_heap::RawHeap, HeapId, HeapObject};

#[derive(Debug, Default)]
pub struct Heap {
    lock: parking_lot::RwLock<()>,
    raw_heap: RawHeap
}

impl Heap {
    pub fn contains(&self, heap_id: &HeapId) -> bool {
        let guard = self.lock.read();
        // Safety:
        // Doesnt access
        unsafe { self.raw_heap.contains(heap_id, guard) }
    }

    /// Safety:
    /// Ensure no concurrent mutable accesses
    pub unsafe fn get<T: 'static>(&self, heap_id: &HeapId) -> Option<&T> {
        let guard = self.lock.read();
        unsafe { self.raw_heap.get(heap_id, guard) }
    }

    /// Safety:
    /// Ensure no concurrent accesses
    pub unsafe fn get_mut<T: 'static>(&self, heap_id: &HeapId) -> Option<&mut T> {
        let guard = self.lock.read();
        unsafe { self.raw_heap.get_mut(heap_id, guard) }
    }

    /// Safety:
    /// Ensure no concurrent accesses
    pub unsafe fn insert(&self, heap_id: HeapId, resource: HeapObject) -> Option<HeapObject> {
        let guard = self.lock.write();
        unsafe { self.raw_heap.insert(heap_id, resource, guard) }
    }
}