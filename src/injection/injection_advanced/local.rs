use std::{any::type_name, sync::Arc};

use crate::{injection::{injection_trait::{Injection, MemoryTarget}, AccessDropper, DeAccessResolver}, memory::{access_map::AccessMap, errors::ResolveError, memory_domain::MemoryDomain, ResourceId}, system::system_metadata::Source};

#[derive(Debug, small_derive_deref::Deref, small_derive_deref::DerefMut)]
pub struct Local<'a, T: Injection> {
    value: T::Item<'a>,
}

impl<'a, T: Injection> Local<'a, T> {
    pub fn new(value: T::Item<'a>) -> Self {
        Self {
            value,
        }
    }
}

impl<T: Injection> AccessDropper for Local<'_, T> {
    fn access_dropper(&self) -> &DeAccessResolver {
        self.value.access_dropper()
    }
}

impl<T: Injection> Injection for Local<'_, T> {
    type Item<'new> = Local<'new, T>;

    fn failed_message() -> String {
        format!("Expected Local Injection: `{}`. Failed with {}", type_name::<T>(), T::failed_message())
    }

    fn create_access_map() -> AccessMap {
        T::create_access_map()
    }

    fn resolve_accesses(access_map: &mut AccessMap, source: Option<&Source>, resource_id: Option<ResourceId>) {
        T::resolve_accesses(access_map, source, resource_id);
    }

    fn select_memory_target() -> MemoryTarget {
        MemoryTarget::Program
    }

    fn retrieve<'a>(memory_domain: &'a Arc<MemoryDomain>, resource_id: Option<&ResourceId>, source: Option<&Source>) -> Result<Self::Item<'a>, ResolveError> {
        Ok(Local::new(T::retrieve(memory_domain, resource_id, source)?))
    }
}

