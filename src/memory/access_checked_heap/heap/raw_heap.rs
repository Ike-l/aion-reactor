use std::cell::UnsafeCell;

use crate::memory::{access_checked_heap::heap::{inner_heap::InnerHeap, HeapObject, }, ResourceId};

#[derive(Debug, Default)]
pub struct RawHeap {
    resources: UnsafeCell<InnerHeap>
}

impl RawHeap {
    /// Safety:
    /// Ensure no concurrent accesses
    unsafe fn get_inner_heap(&self) -> &InnerHeap {
        unsafe { & *self.resources.get() }
    }

    /// Safety:
    /// Ensure no concurrent accesses
    unsafe fn get_mut_inner_heap(&self) -> &mut InnerHeap {
        unsafe { &mut *self.resources.get() }
    }

    /// Safety:
    /// Ensure no concurrent accesses
    pub unsafe fn get<T: 'static>(&self, resource_id: &ResourceId, _guard: parking_lot::RwLockReadGuard<()>) -> Option<&T> {
        unsafe { self.get_inner_heap().get(resource_id) }        
    }

    /// Safety:
    /// Ensure no concurrent accesses
    pub unsafe fn get_mut<T: 'static>(&self, resource_id: &ResourceId, _guard: parking_lot::RwLockReadGuard<()>) -> Option<&mut T> {
        unsafe { self.get_inner_heap().get_mut(resource_id) }
    }

    pub unsafe fn insert(&self, resource_id: ResourceId, resource: HeapObject, _guard: parking_lot::RwLockWriteGuard<()>) -> Option<HeapObject> {
        unsafe { self.get_mut_inner_heap().insert(resource_id, resource) }
    }
}