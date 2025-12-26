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
        // Does this need to be unsafe?
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
    /// Ensure no concurrent mutable accesses
    pub unsafe fn get_mut<T: 'static>(&self, heap_id: &HeapId) -> Option<&mut T> {
        let guard = self.lock.read();
        unsafe { self.raw_heap.get_mut(heap_id, guard) }
    }

    /// Safety:
    /// Ensure no concurrent accesses
    pub unsafe fn insert(&self, heap_id: HeapId, resource: HeapObject) -> Option<HeapObject> {
        let guard = self.lock.write();

        // Safety:
        // since no concurrent accesses
        unsafe { self.raw_heap.insert(heap_id, resource, guard) }
    }
}

// would want a test to show no race conditions on inserts / _

#[cfg(test)]
mod heap_tests {
    use crate::prelude::{Heap, HeapId, HeapObject, Id};

    #[test]
    fn insert_and_contains() {
        let heap = Heap::default();
        let id = HeapId::Label(Id::from("foo"));
        assert!(!heap.contains(&id));
        assert!(unsafe { heap.insert(id.clone(), HeapObject::dummy(100)) }.is_none());
        assert!(unsafe { heap.insert(id.clone(), HeapObject::dummy(101)) }.is_some());
        assert!(heap.contains(&id));
    }

    #[test]
    fn get() {
        let heap = Heap::default();
        let id = HeapId::Label(Id::from("foo"));
        assert!(!heap.contains(&id));
        assert!(unsafe { heap.get::<i32>(&id) }.is_none());
        assert!(unsafe { heap.insert(id.clone(), HeapObject::dummy(100)) }.is_none());
        assert!(heap.contains(&id));
        assert_eq!(unsafe { heap.get::<i32>(&id) }, Some(&100));
    }

    #[test]
    fn get_mut() {
        let heap = Heap::default();
        let id = HeapId::Label(Id::from("foo"));
        assert!(!heap.contains(&id));
        assert!(unsafe { heap.get_mut::<i32>(&id) }.is_none());
        assert!(unsafe { heap.insert(id.clone(), HeapObject::dummy(100)) }.is_none());
        assert!(heap.contains(&id));
        assert_eq!(unsafe { heap.get_mut::<i32>(&id) }, Some(&mut 100));
    }
}