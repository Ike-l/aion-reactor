use std::sync::Arc;

use crate::{id::Id, memory::{memory_domain::MemoryDomain, program_memory_map::{inner_program_memory_map::Key, raw_program_memory_map::RawProgramMemoryMap}}};

pub mod inner_program_memory_map;
pub mod raw_program_memory_map;

#[derive(Debug, Default)]
pub struct ProgramMemoryMap {
    lock: parking_lot::RwLock<()>,
    raw_program_memory_map: RawProgramMemoryMap,
}

impl ProgramMemoryMap {
    pub fn get(&self, id: &Id, key: Option<&Key>) -> Option<&Arc<MemoryDomain>> {
        let guard = self.lock.read(); 
        // Safety:
        // inherent since no `get_mut`
        unsafe { self.raw_program_memory_map.get(id, key, guard) }
    }

    pub fn get_or_default(&self, id: Id, key: Option<&Key>) -> &Arc<MemoryDomain> {
        if self.get(&id, key).is_none() {
            self.insert(id.clone(), Arc::new(MemoryDomain::new()), key.cloned());
        }

        self.get(&id, key).as_ref().unwrap()
    }

    pub fn insert(&self, program_id: Id, memory_domain: Arc<MemoryDomain>, key: Option<Key>) -> bool {
        let guard = self.lock.write();
        // Safety:
        // Tracked through "lock"
        unsafe { self.raw_program_memory_map.insert(program_id, memory_domain, key, guard) }
    }

    /// Safety:
    /// Do not free the lock before finished using memory map
    pub unsafe fn get_write_program_memory(&self) -> (parking_lot::lock_api::RwLockWriteGuard<'_, parking_lot::RawRwLock, ()>, &RawProgramMemoryMap) {
        (self.lock.write(), &self.raw_program_memory_map)
    }

    pub fn consume(self) -> impl Iterator<Item = (Option<Key>, Id, Arc<MemoryDomain>)> {
        self.raw_program_memory_map.consume()
    }
}