use std::pin::Pin;

use crate::{id::Id, memory::Memory, system::{system_metadata::Source, system_result::SystemResult}};

pub type StoredAsyncSystem = Box<dyn AsyncSystem>;

pub trait AsyncSystem: Send + Sync {
    fn run<'a>(
        &'a mut self,
        memory: &'a Memory,
        program_id: Option<&Id>, 
        source: Option<&Source>
    ) -> Pin<Box<dyn Future<Output = Option<SystemResult>> + 'a>>;

    fn ok_resources(&self, memory: &Memory, program_id: Option<&Id>, source: &Source) -> Option<bool>;
}
