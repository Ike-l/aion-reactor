use std::{any::{type_name, TypeId}, sync::Arc};

use crate::{id::Id, injection::{injection_trait::Injection, resolve, AccessDeResolver, AccessDropper}, memory::{access_checked_heap::heap::HeapId, access_map::AccessMap, errors::ResolveError, memory_domain::MemoryDomain, Memory, ResourceId}};

pub struct Shared<'a, T> {
    pub value: &'a T,
    dropper: AccessDeResolver
}

impl<'a, T: 'static> Shared<'a, T> {
    pub fn new(value: &'a T, dropper: AccessDeResolver) -> Self {
        Self {
            value,
            dropper
        }
    }
}

impl<T> AccessDropper for Shared<'_, T> {
    fn access_dropper(&self) -> &AccessDeResolver {
        &self.dropper
    }
}

impl<T: 'static> Injection for Shared<'_, T> {
    type Item<'new> = Shared<'new, T>;

    fn failed_message() -> String {
        format!("Expected Resource: `{}`", type_name::<T>())
    }

    fn resolve_accesses(access_map: &mut AccessMap) {
        match access_map {
            AccessMap::Heap(access_map) => access_map.access_shared(TypeId::of::<T>()).unwrap()
        }
    }
    
    fn resolve<'a>(memory: &'a Memory, program_id: Option<Id>, resource_id: Option<ResourceId>) -> anyhow::Result<Result<Self::Item<'a>, ResolveError>> {
        resolve!(memory, program_id, resource_id)
    }

    fn retrieve<'a>(memory_domain: &'a Arc<MemoryDomain>, resource_id: Option<ResourceId>) -> Result<Self::Item<'a>, ResolveError> {
        let access = resource_id.unwrap_or(ResourceId::from(HeapId::from(TypeId::of::<T>())));
        let (result, access_map) = memory_domain.get_shared::<T>(access)?;

        let dropper = AccessDeResolver::new(Arc::clone(memory_domain), access_map);
        let shared = Shared::new(result, dropper);

        Ok(shared)
    }
}

