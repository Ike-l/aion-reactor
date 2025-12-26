use std::{cell::UnsafeCell, sync::Arc};

use crate::{ids::program_id::ProgramId, memory::{memory_domain::MemoryDomain, program_memory_map::inner_program_memory_map::{InnerProgramMemoryMap, ProgramKey}}};

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
    pub unsafe fn get(&self, program_id: &ProgramId, key: Option<&ProgramKey>, _guard: parking_lot::RwLockReadGuard<()>) -> Option<&Arc<MemoryDomain>> {
        unsafe { self.get_inner_heap().get(program_id, key) }
    }

    /// Safety:
    /// Ensure the lock actually guards the memory
    pub unsafe fn get_with_write(&self, program_id: &ProgramId, key: Option<&ProgramKey>, _guard: &parking_lot::RwLockWriteGuard<()>) -> Option<&Arc<MemoryDomain>> {
        unsafe { self.get_inner_heap().get(program_id, key) }
    }

    /// Safety restrain satisfied because it will only insert if it doesn't exist already
    /// Safety:
    /// Ensure no concurrent accesses
    pub unsafe fn insert(&self, program_id: ProgramId, memory_domain: Arc<MemoryDomain>, key: Option<ProgramKey>, _guard: parking_lot::RwLockWriteGuard<()>) -> bool {
        let inner_program_map = unsafe { self.get_mut_inner_heap() };
        if !inner_program_map.contains(&program_id) {
            inner_program_map.insert(program_id, memory_domain, key);
            return true;
        }
        
        false
    }

    pub fn consume(self) -> impl Iterator<Item = (Option<ProgramKey>, ProgramId, Arc<MemoryDomain>)> {
        self.inner_program_memory_map.into_inner().consume()
    }
}