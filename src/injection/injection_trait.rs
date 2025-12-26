use std::sync::Arc;

use crate::prelude::{AccessDropper, AccessMap, MemoryDomain, ResolveError, ResourceId, SystemId};

#[derive(Debug)]
pub enum MemoryTarget {
    Program,
    Global
}

pub trait Injection {
    type Item<'new>: AccessDropper;
    
    fn create_access_map() -> AccessMap;

    fn resolve_accesses(access_map: &mut AccessMap, system_id: Option<&SystemId>, resource_id: Option<ResourceId>);
    
    // fn create_and_resolve_access_map(source: Option<&Source>, resource_id: Option<ResourceId>) -> AccessMap {
    //     let mut access_map = Self::create_access_map();
    //     Self::resolve_accesses(&mut access_map, source, resource_id);
    //     access_map
    // }

    fn failed_message() -> String;
    
    fn retrieve<'a>(memory_domain: &'a Arc<MemoryDomain>, resource_id: Option<&ResourceId>, system_id: Option<&SystemId>) -> Result<Self::Item<'a>, ResolveError>;

    fn select_memory_target() -> MemoryTarget {
        MemoryTarget::Program
    }
}
