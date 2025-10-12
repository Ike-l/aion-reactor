use std::sync::Arc;

use crate::{id::Id, injection::AccessDropper, memory::{access_checked_resource_map::{access::access_map::AccessMap, AccessCheckedResourceMap, ResolveError}, Memory} };

pub enum MemoryTarget {
    Program,
    Global
}

pub trait Injection {
    type Item<'new>: AccessDropper;
    
    fn resolve_accesses(access_map: &mut AccessMap);
    
    fn failed_message() -> String;
    
    fn resolve<'a>(memory: &'a Memory, program_id: Id) -> anyhow::Result<Result<Self::Item<'a>, ResolveError>>;
    
    fn retrieve<'a>(resource_map: &'a Arc<AccessCheckedResourceMap>) -> Result<Self::Item<'a>, ResolveError>;


    fn select_memory_target() -> MemoryTarget {
        MemoryTarget::Global
    }
}
