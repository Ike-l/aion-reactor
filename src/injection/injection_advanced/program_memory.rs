use std::{fmt::{Debug, Display}, sync::Arc};

use crate::prelude::{AccessDropper, AccessMap, DeAccessResolver, Injection, MemoryDomain, ReservationAccessMap, ResolveError, ResourceId, SystemId};

#[derive(small_derive_deref::Deref, small_derive_deref::DerefMut)]
pub struct ProgramMemory {
    #[DerefTarget]
    #[DerefMutTarget]
    value: Arc<MemoryDomain>,
    dropper: DeAccessResolver
}

impl Debug for ProgramMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }   
}

impl Display for ProgramMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }   
}

impl ProgramMemory {
    pub fn new(value: Arc<MemoryDomain>, dropper: DeAccessResolver) -> Self {
        Self {
            value,
            dropper
        }
    }
}

impl AccessDropper for ProgramMemory {
    fn access_dropper(&self) -> &DeAccessResolver {
        &self.dropper
    }
}

impl Injection for ProgramMemory {
    type Item<'new> = ProgramMemory;

    fn failed_message() -> String {
        unreachable!()
    }

    fn create_access_map() -> AccessMap {
        AccessMap::Heap(ReservationAccessMap::default())
    }

    fn resolve_accesses(_access_map: &mut AccessMap, _system_id: Option<&SystemId>, _resource_id: Option<ResourceId>) {}

    fn retrieve<'a>(memory_domain: &'a Arc<MemoryDomain>, _resource_id: Option<&ResourceId>, _system_id: Option<&SystemId>) -> Result<Self::Item<'a>, ResolveError> {
        let dropper = DeAccessResolver::new(Arc::clone(memory_domain), Self::create_access_map());
        let get_system_id = ProgramMemory::new(Arc::clone(memory_domain), dropper);

        Ok(get_system_id)
    }
}
