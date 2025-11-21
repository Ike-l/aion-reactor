use crate::{id::Id, memory::{Memory, errors::ReservationError, program_memory_map::inner_program_memory_map::Key}, system::{async_system::{AsyncSystem, StoredAsyncSystem, into_async_system::IntoAsyncSystem}, sync_system::{StoredSyncSystem, SyncSystem, into_sync_system::IntoSyncSystem}, system_metadata::Source}};

pub mod system_metadata;
pub mod stored_system;
pub mod system_status;
pub mod sync_system;
pub mod async_system;
pub mod system_result;
pub mod system_cell;

use std::marker::PhantomData;

pub struct FunctionSystem<Input, F> {
    f: F,
    marker: PhantomData<fn() -> Input>
}

pub enum System {
    Sync(StoredSyncSystem),
    Async(StoredAsyncSystem)
}

impl System {
    pub fn new_sync<T, S, I>(system: T) -> Self where T: IntoSyncSystem<I, System = S>, S: SyncSystem + 'static {
        Self::Sync(Box::new(system.into_system()))
    }

    pub fn new_async<T, S, I>(system: T) -> Self where T: IntoAsyncSystem<I, System = S>, S: AsyncSystem + 'static {
        Self::Async(Box::new(system.into_system()))
    }

    // True if success, False if fail, None if program_id is Invalid
    pub fn ok_resources(&self, memory: &Memory, program_id: Option<&Id>, source: Option<&Source>, key: Option<&Key>) -> Option<bool> {
        match self {
            Self::Async(system) => system.ok_resources(memory, program_id, source, key),
            Self::Sync(system) => system.ok_resources(memory, program_id, source, key)
        }
    }

    // True if success, False if fail, None if program_id is Invalid
    pub fn ok_accesses(&self, memory: &Memory, program_id: Option<&Id>, source: Option<&Source>, key: Option<&Key>) -> Option<bool> {
        match self {
            System::Sync(sync_system) => sync_system.ok_accesses(memory, program_id, source, key),
            System::Async(async_system) => async_system.ok_accesses(memory, program_id, source, key),
        }
    }

    // True if success, False if fail, None if program_id is Invalid
    pub fn reserve_accesses(&self, memory: &Memory, program_id: Option<&Id>, source: Source, key: Option<&Key>) -> Option<Result<(), ReservationError>> {
        match self {
            System::Sync(sync_system) => sync_system.reserve_accesses(memory, program_id, source, key),
            System::Async(async_system) => async_system.reserve_accesses(memory, program_id, source, key),
        }
    }
}