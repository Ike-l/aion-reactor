use std::cell::UnsafeCell;

use crate::memory::access_checked_heap::heap::{inner_heap::InnerHeap, HeapId, HeapObject };

#[derive(Debug, Default)]
pub struct RawHeap {
    resources: UnsafeCell<InnerHeap>
}

unsafe impl Send for RawHeap {}
unsafe impl Sync for RawHeap {}

impl RawHeap {
    /// Safety:
    /// Ensure no mutable concurrent accesses
    unsafe fn get_inner_heap(&self) -> &InnerHeap {
        unsafe { & *self.resources.get() }
    }

    /// Safety:
    /// Ensure no mutable concurrent accesses
    unsafe fn get_mut_inner_heap(&self) -> &mut InnerHeap {
        unsafe { &mut *self.resources.get() }
    }

    /// Does this need to be unsafe?
    /// Safety: 
    /// Ensure no mutable concurrent accesses
    pub unsafe fn contains(&self, heap_id: &HeapId, _guard: parking_lot::RwLockReadGuard<()>) -> bool {
        unsafe { self.get_inner_heap().contains(heap_id) }
    }

    /// Safety:
    /// Ensure no mutable concurrent accesses
    pub unsafe fn get<T: 'static>(&self, heap_id: &HeapId, _guard: parking_lot::RwLockReadGuard<()>) -> Option<&T> {
        unsafe { self.get_inner_heap().get(heap_id) }        
    }

    /// Safety:
    /// Ensure no mutable concurrent accesses
    pub unsafe fn get_mut<T: 'static>(&self, heap_id: &HeapId, _guard: parking_lot::RwLockReadGuard<()>) -> Option<&mut T> {
        unsafe { self.get_inner_heap().get_mut(heap_id) }
    }

    /// Safety:
    /// Ensure no access overwritten
    pub unsafe fn insert(&self, heap_id: HeapId, heap_object: HeapObject, _guard: parking_lot::RwLockWriteGuard<()>) -> Option<HeapObject> {
        unsafe { self.get_mut_inner_heap().insert(heap_id, heap_object) }
    }
}