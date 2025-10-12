use std::{any::{type_name, TypeId}, sync::Arc};

use crate::{id::Id, injection::{injection_trait::Injection, resolve, retrieve, AccessDeResolver, AccessDropper}, memory::{access_checked_resource_map::{access::access_map::AccessMap, resource::ResourceId, AccessCheckedResourceMap, ResolveError}, Memory}};

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
        let _ = access_map.access_shared(TypeId::of::<T>()).unwrap();
    }
    
    fn resolve<'a>(memory: &'a Memory, program_id: Option<Id>, resource_id: Option<ResourceId>) -> anyhow::Result<Result<Self::Item<'a>, ResolveError>> {
        resolve!(memory, program_id, resource_id)
    }

    fn retrieve<'a>(resource_map: &'a Arc<AccessCheckedResourceMap>, resource_id: Option<ResourceId>) -> Result<Self::Item<'a>, ResolveError> {
        let r = resource_map.get_shared::<T>(resource_id.unwrap_or(ResourceId::RawTypeId(TypeId::of::<T>())))?;
        let dropper = retrieve!(resource_map);
        let shared = Shared::new(r, dropper);

        Ok(shared)
    }
}

