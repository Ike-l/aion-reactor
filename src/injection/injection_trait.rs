use std::sync::Arc;

use crate::{id::Id, injection::AccessDropper, memory::{access_map::AccessMap, errors::ResolveError, memory_domain::MemoryDomain, Memory, ResourceId}, system::system_metadata::Source };

pub enum MemoryTarget {
    Program,
    Global
}

pub trait Injection {
    type Item<'new>: AccessDropper;
    
    fn create_access_map() -> AccessMap;

    fn resolve_accesses(access_map: &mut AccessMap, source: Option<&Source>, resource_id: Option<ResourceId>);
    
    fn create_and_resolve_access_map(source: Option<&Source>, resource_id: Option<ResourceId>) -> AccessMap {
        let mut access_map = Self::create_access_map();
        Self::resolve_accesses(&mut access_map, source, resource_id);
        access_map
    }

    fn failed_message() -> String;
    
    fn resolve<'a>(memory: &'a Memory, program_id: Option<&Id>, resource_id: Option<&ResourceId>, source: Option<&Source>) -> anyhow::Result<Result<Self::Item<'a>, ResolveError>>;
    
    fn retrieve<'a>(memory_domain: &'a Arc<MemoryDomain>, resource_id: Option<&ResourceId>, source: Option<&Source>) -> Result<Self::Item<'a>, ResolveError>;

    fn select_memory_target() -> MemoryTarget {
        MemoryTarget::Global
    }
}
