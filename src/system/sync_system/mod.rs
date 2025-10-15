use crate::{id::Id, memory::Memory, system::{system_metadata::Source, system_result::SystemResult}};

pub type StoredSyncSystem = Box<dyn SyncSystem>;

pub trait SyncSystem: Send + Sync {
    fn run(
        &mut self,
        memory: &Memory,
        program_id: Option<&Id>, 
        source: Option<&Source>
    ) -> Option<SystemResult>;

    fn ok_resources(&self, memory: &Memory, program_id: Option<&Id>, source: &Source) -> Option<bool>;

    fn reserve_accesses(&self, memory: &Memory, program_id: Option<&Id>, source: &Source) -> Option<bool>;
}
