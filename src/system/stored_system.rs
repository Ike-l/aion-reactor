use std::sync::Mutex;

use crate::{id::Id, memory::Memory, system::{system_metadata::Source, system_status::SystemStatus, System}};


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

    pub fn ok_resources(&self, memory: &Memory, program_id: Option<&Id>, source: Option<&Source>) -> Option<bool> {
        self.system.as_ref().expect("System").ok_resources(memory, program_id, source)
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

