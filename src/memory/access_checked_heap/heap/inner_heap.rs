use std::collections::HashMap;

use crate::memory::access_checked_heap::heap::{HeapId, HeapObject};

#[derive(Debug, Default)]
pub struct InnerHeap {
    resources: HashMap<HeapId, HeapObject>
}

impl InnerHeap {
    /// Safety:
    /// Ensure no concurrent accesses
    pub unsafe fn get<T: 'static>(&self, resource_id: &HeapId) -> Option<&T> {
        unsafe {
            self.resources
                .get(resource_id)
                .map(|cell| & *cell.0.get())
                .and_then(|boxed| boxed.downcast_ref::<T>())
        }
    }

    /// Safety:
    /// Ensure no concurrent accesses
    pub unsafe fn get_mut<T: 'static>(&self, resource_id: &HeapId) -> Option<&mut T> {
        unsafe {
            self.resources
                .get(resource_id)
                .map(|cell| &mut *cell.0.get())
                .and_then(|boxed| boxed.downcast_mut::<T>())
        }
    }

    pub unsafe fn insert(&mut self, resource_id: HeapId, resource: HeapObject) -> Option<HeapObject> {
        self.resources.insert(resource_id, resource)
    }
}