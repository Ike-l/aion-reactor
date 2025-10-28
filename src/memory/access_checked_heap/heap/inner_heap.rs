use std::collections::HashMap;

use crate::memory::access_checked_heap::heap::{HeapId, HeapObject};

#[derive(Debug, Default)]
pub struct InnerHeap {
    resources: HashMap<HeapId, HeapObject>
}

impl InnerHeap {
    pub fn contains(&self, heap_id: &HeapId) -> bool {
        self.resources.contains_key(heap_id)
    }

    /// Safety:
    /// Ensure no mutable concurrent accesses
    pub unsafe fn get<T: 'static>(&self, heap_id: &HeapId) -> Option<&T> {
        unsafe {
            self.resources
                .get(heap_id)
                .map(|cell| & *cell.0.get())
                .and_then(|boxed| boxed.downcast_ref::<T>())
        }
    }

    /// Safety:
    /// Ensure no mutable concurrent accesses
    pub unsafe fn get_mut<T: 'static>(&self, heap_id: &HeapId) -> Option<&mut T> {
        unsafe {
            self.resources
                .get(heap_id)
                .map(|cell| &mut *cell.0.get())
                .and_then(|boxed| boxed.downcast_mut::<T>())
        }
    }

    /// Safety:
    /// Ensure no concurrent accesses
    pub unsafe fn insert(&mut self, heap_id: HeapId, heap_object: HeapObject) -> Option<HeapObject> {
        self.resources.insert(heap_id, heap_object)
    }
}

#[cfg(test)]
mod inner_heap_tests {
    use crate::{id::Id, memory::access_checked_heap::heap::{HeapId, HeapObject, inner_heap::InnerHeap, raw_heap_object::RawHeapObject}};

    #[test]
    fn insert_and_contains() {
        let mut heap = InnerHeap::default();
        let id = HeapId::Label(Id("foo".to_string()));
        assert!(!heap.contains(&id));
        assert!(unsafe { heap.insert(id.clone(), HeapObject(RawHeapObject::new(Box::new(100) as Box<i32>))) }.is_none());
        assert!(unsafe { heap.insert(id.clone(), HeapObject(RawHeapObject::new(Box::new(101) as Box<i32>))) }.is_some());
        assert!(heap.contains(&id));
    }

    #[test]
    fn get() {
        let mut heap = InnerHeap::default();
        let id = HeapId::Label(Id("foo".to_string()));
        assert!(!heap.contains(&id));
        assert!(unsafe { heap.insert(id.clone(), HeapObject(RawHeapObject::new(Box::new(100) as Box<i32>))) }.is_none());
        assert!(heap.contains(&id));
        assert_eq!(unsafe { heap.get::<i32>(&id) }, Some(&100));
    }

    #[test]
    fn get_mut() {
        let mut heap = InnerHeap::default();
        let id = HeapId::Label(Id("foo".to_string()));
        assert!(!heap.contains(&id));
        assert!(unsafe { heap.insert(id.clone(), HeapObject(RawHeapObject::new(Box::new(100) as Box<i32>))) }.is_none());
        assert!(heap.contains(&id));
        assert_eq!(unsafe { heap.get_mut::<i32>(&id) }, Some(&mut 100));
    }
}