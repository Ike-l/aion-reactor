use std::sync::Arc;

use crate::{id::Id, injection::AccessDropper, memory::{access_checked_heap::access::access_map::HeapAccessMap, errors::ResolveError, memory_domain::MemoryDomain, Memory, ResourceId} };

pub enum MemoryTarget {
    Program,
    Global
}

pub trait Injection {
    type Item<'new>: AccessDropper;
    
    fn resolve_accesses(access_map: &mut HeapAccessMap);
    
    fn failed_message() -> String;
    
    fn resolve<'a>(memory: &'a Memory, program_id: Option<Id>, resource_id: Option<ResourceId>) -> anyhow::Result<Result<Self::Item<'a>, ResolveError>>;
    
    fn retrieve<'a>(memory_domain: &'a Arc<MemoryDomain>, resource_id: Option<ResourceId>) -> Result<Self::Item<'a>, ResolveError>;

    fn select_memory_target() -> MemoryTarget {
        MemoryTarget::Global
    }
}
