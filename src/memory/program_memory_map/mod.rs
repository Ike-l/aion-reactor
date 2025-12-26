use std::sync::Arc;

use crate::{ids::{program_id::ProgramId, system_id::SystemId}, memory::{errors::ReservationError, memory_domain::MemoryDomain, program_memory_map::{inner_program_memory_map::ProgramKey, raw_program_memory_map::RawProgramMemoryMap}}};

pub mod inner_program_memory_map;
pub mod raw_program_memory_map;

#[derive(Debug, Default)]
pub struct ProgramMemoryMap {
    lock: parking_lot::RwLock<()>,
    raw_program_memory_map: RawProgramMemoryMap,
}

impl ProgramMemoryMap {
    pub fn get(&self, id: &ProgramId, key: Option<&ProgramKey>) -> Option<&Arc<MemoryDomain>> {
        let guard = self.lock.read(); 
        // Safety:
        // inherent since no `get_mut`
        unsafe { self.raw_program_memory_map.get(id, key, guard) }
    }

    pub fn get_or_default(&self, id: ProgramId, key: Option<&ProgramKey>) -> &Arc<MemoryDomain> {
        if self.get(&id, key).is_none() {
            self.insert(id.clone(), Arc::new(MemoryDomain::new()), key.cloned());
        }

        self.get(&id, key).as_ref().unwrap()
    }

    pub fn insert(&self, program_id: ProgramId, memory_domain: Arc<MemoryDomain>, key: Option<ProgramKey>) -> bool {
        let guard = self.lock.write();
        // Safety:
        // Tracked through "lock"
        unsafe { self.raw_program_memory_map.insert(program_id, memory_domain, key, guard) }
    }

    // can refactor checked_reservations over an abstract builder thing
    pub fn atomic_reservations(&self, other: Self, system_id: &SystemId) -> Result<(), ReservationError> {
        let guard = self.lock.write();

        let other = other.raw_program_memory_map.consume().collect::<Vec<_>>();

        for (key, program_id, memory_domain) in other.iter() {
            // Safety:
            // The guard is held for the entire function
            if let Some(program_memory) = unsafe { self.raw_program_memory_map.get_with_write(program_id, key.as_ref(), &guard) } {
                match program_memory.ok_reservation_self(memory_domain.as_ref(), Some(system_id)) {
                    Some(reservation_error) => return Err(reservation_error),
                    None => ()
                }
            }
        }
        
        for (key, program_id, memory_domain) in other {
            // Safety:
            // The guard is held for the entire function
            if let Some(program_memory) = unsafe { self.raw_program_memory_map.get_with_write(&program_id, key.as_ref(), &guard) } {
                match program_memory.reserve_accesses_self(system_id.clone(), Arc::into_inner(memory_domain).unwrap()) {
                    Err(_) => unreachable!(),
                    Ok(_) => ()
                }
            }
        }

        Ok(())
    }
}