use std::{cell::UnsafeCell, sync::Arc};

use crate::{id::Id, memory::{memory_domain::MemoryDomain, program_memory_map::inner_program_memory_map::InnerProgramMemoryMap}};

#[derive(Debug, Default)]
pub struct RawProgramMemoryMap {
    inner_program_memory_map: UnsafeCell<InnerProgramMemoryMap>
}

unsafe impl Send for RawProgramMemoryMap {}
unsafe impl Sync for RawProgramMemoryMap {}

impl RawProgramMemoryMap {
    /// Safety:
    /// Ensure no concurrent mutable accesses
    unsafe fn get_inner_heap(&self) -> &InnerProgramMemoryMap {
        unsafe { & *self.inner_program_memory_map.get() }
    }

    /// Safety:
    /// Ensure no concurrent mutable accesses
    unsafe fn get_mut_inner_heap(&self) -> &mut InnerProgramMemoryMap {
        unsafe { &mut *self.inner_program_memory_map.get() }
    }

    /// Safety:
    /// Ensure no concurrent mutable accesses
    pub unsafe fn get(&self, program_id: &Id, _guard: parking_lot::RwLockReadGuard<()>) -> Option<&Arc<MemoryDomain>> {
        unsafe { self.get_inner_heap().get(program_id) }
    }

    /// Safety restrain satisfied because it will only insert if it doesn't exist already
    /// Safety:
    /// Ensure no concurrent accesses
    pub unsafe fn insert(&self, program_id: Id, memory_domain: Arc<MemoryDomain>, _guard: parking_lot::RwLockWriteGuard<()>) -> bool {
        let inner_program_map = unsafe { self.get_mut_inner_heap() };
        if !inner_program_map.contains(&program_id) {
            inner_program_map.insert(program_id, memory_domain);
            return true;
        }
        
        false
    }
}