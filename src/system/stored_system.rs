use std::sync::Mutex;

use crate::{id::Id, memory::{Memory, errors::ReservationError, program_memory_map::inner_program_memory_map::Key}, system::{System, system_metadata::Source, system_status::SystemStatus}};


pub struct StoredSystem {
    system: Option<System>,
    status: Mutex<SystemStatus>
}

impl StoredSystem {
    pub fn new(system: System) -> Self {
        Self {
            system: Some(system),
            status: Mutex::new(SystemStatus::Ready)
        }
    }

    pub fn ok_resources(&self, memory: &Memory, program_id: Option<&Id>, source: Option<&Source>, key: Option<&Key>) -> Option<bool> {
        self.system.as_ref().expect("System").ok_resources(memory, program_id, source, key)
    }

    pub fn ok_accesses(&self, memory: &Memory, program_id: Option<&Id>, source: Option<&Source>, key: Option<&Key>) -> Option<bool> {
        self.system.as_ref().expect("System").ok_accesses(memory, program_id, source, key)
    }

    pub fn reserve_accesses(&self, memory: &Memory, program_id: Option<&Id>, source: Source, key: Option<&Key>) -> Option<Result<(), ReservationError>> {
        self.system.as_ref().expect("System").reserve_accesses(memory, program_id, source, key)
    }

    pub fn status(&self) -> &Mutex<SystemStatus> {
        &self.status
    }

    pub fn take_system(&mut self) -> Option<System> {
        self.system.take()
    }

    pub fn insert_system(&mut self, system: System) -> Option<System> {
        self.system.replace(system)
    }
}

