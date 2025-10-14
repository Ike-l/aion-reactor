use std::{cell::UnsafeCell, pin::Pin, sync::Mutex};

use crate::{id::Id, memory::{Memory, ResourceId}, system::system_status::SystemStatus};

pub trait SyncSystem: Send + Sync {
    fn run(
        &mut self,
        memory: &Memory,
        program_id: Option<&Id>, 
        source: Option<&ResourceId>
    ) -> Option<SystemResult>;
}

type StoredSyncSystem = Box<dyn SyncSystem>;

#[derive(Debug)]
pub enum SystemEvent {
    NoEvent,
}

#[derive(Debug)]
pub enum SystemResult {
    Event(SystemEvent),
    Error(anyhow::Error)
}

pub trait AsyncSystem: Send + Sync {
    fn run<'a>(
        &'a mut self,
        memory: &'a Memory,
        program_id: Option<&Id>, 
        source: Option<&ResourceId>
    ) -> Pin<Box<dyn Future<Output = Option<SystemResult>> + 'a>>;
}

type StoredAsyncSystem = Box<dyn AsyncSystem>;

pub enum System {
    Sync(StoredSyncSystem),
    Async(StoredAsyncSystem)
}

impl System {
    // True if success, False if fail, None if program_id is Invalid
    pub fn ok_resources(&self, memory: &Memory, program_id: Option<&Id>) -> Option<bool> {
        match self {
            Self::Async(system) => todo!(),
            Self::Sync(system) => todo!()
        }
    }

    // True if success, False if fail, None if program_id is Invalid
    pub fn ok_accesses(&self, memory: &Memory, program_id: Option<&Id>) -> Option<bool> {
        match self {
            System::Sync(sync_system) => todo!(),
            System::Async(async_system) => todo!(),
        }
    }

    // True if success, False if fail, None if program_id is Invalid
    pub fn reserve_accesses(&self, memory: &Memory, program_id: Option<&Id>, source: &ResourceId) -> Option<bool> {
        match self {
            System::Sync(sync_system) => todo!(),
            System::Async(async_system) => todo!(),
        }
    }
}
pub struct StoredSystem {
    system: Option<System>,
    status: Mutex<SystemStatus>
}

impl StoredSystem {
    pub fn ok_resources(&self, memory: &Memory, program_id: Option<&Id>) -> Option<bool> {
        self.system.as_ref().expect("System").ok_resources(memory, program_id)
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

#[derive(Debug)]
pub struct SystemCell(UnsafeCell<System>);

impl SystemCell {
    pub fn new(system: System) -> Self {
        Self(UnsafeCell::new(system))
    }

    pub fn consume(self) -> System {
        self.0.into_inner()
    }

    /// Safety:
    /// Ensure only 1 reference to System
    pub unsafe fn get(&self) -> &mut System {
        unsafe { &mut *self.0.get() }
    }
}

unsafe impl Send for SystemCell {}
unsafe impl Sync for SystemCell {}