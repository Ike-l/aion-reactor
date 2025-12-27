use std::sync::Mutex;

use crate::prelude::{ProgramKey, Memory, ProgramId, ReservationError, System, SystemId, SystemStatus};

#[derive(Debug)]
pub enum StoredSystemError {
    MissingSystem
}

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

    pub fn ok_resources(&self, memory: &Memory, program_id: Option<&ProgramId>, source: Option<&SystemId>, key: Option<&ProgramKey>) -> Result<Option<bool>, StoredSystemError> {
        match self.system.as_ref() {
            Some(system) => Ok(system.ok_resources(memory, program_id, source, key)),
            None => return Err(StoredSystemError::MissingSystem),
        }
    }

    pub fn ok_accesses(&self, memory: &Memory, program_id: Option<&ProgramId>, source: Option<&SystemId>, key: Option<&ProgramKey>) -> Result<Option<bool>, StoredSystemError> {
        match self.system.as_ref() {
            Some(system) => Ok(system.ok_accesses(memory, program_id, source, key)),
            None => return Err(StoredSystemError::MissingSystem),
        }
    }

    pub fn reserve_accesses(&self, memory: &Memory, program_id: Option<&ProgramId>, source: SystemId, key: Option<&ProgramKey>) -> Result<Option<Result<(), ReservationError>>, StoredSystemError> {
        match self.system.as_ref() {
            Some(system) => Ok(system.reserve_accesses(memory, program_id, source, key)),
            None => return Err(StoredSystemError::MissingSystem),
        }
    }

    pub fn has_system(&self) -> bool {
        self.system.is_some()
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

