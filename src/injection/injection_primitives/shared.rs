use std::{any::{type_name, TypeId}, sync::Arc};

use crate::{injection::{AccessDropper, DeAccessResolver, injection_trait::Injection}, memory::{ResourceId, access_checked_heap::{heap::HeapId, reservation_access_map::ReservationAccessMap}, access_map::{Access, AccessMap}, errors::ResolveError, memory_domain::MemoryDomain}, system::system_metadata::Source};

#[derive(Debug, small_derive_deref::Deref, small_derive_deref::DerefMut)]
pub struct Shared<'a, T> {
    #[DerefTarget]
    #[DerefMutTarget]
    value: &'a T,
    dropper: DeAccessResolver
}

impl<'a, T: 'static> Shared<'a, T> {
    pub fn new(value: &'a T, dropper: DeAccessResolver) -> Self {
        Self {
            value,
            dropper
        }
    }
}

impl<T> AccessDropper for Shared<'_, T> {
    fn access_dropper(&self) -> &DeAccessResolver {
        &self.dropper
    }
}

impl<T: 'static> Injection for Shared<'_, T> {
    type Item<'new> = Shared<'new, T>;

    fn failed_message() -> String {
        format!("Expected Resource: `{}`", type_name::<T>())
    }

    fn create_access_map() -> AccessMap {
        AccessMap::Heap(ReservationAccessMap::default())
    }

    fn resolve_accesses(access_map: &mut AccessMap, source: Option<&Source>, resource_id: Option<ResourceId>) {
        match (access_map, resource_id.unwrap_or(ResourceId::Heap(HeapId::RawType(TypeId::of::<T>())))) {
            (AccessMap::Heap(access_map), ResourceId::Heap(heap_id)) => access_map.do_access(heap_id, source, Access::Shared(1)).unwrap()
        }
    }

    fn retrieve<'a>(memory_domain: &'a Arc<MemoryDomain>, resource_id: Option<&ResourceId>, source: Option<&Source>) -> Result<Self::Item<'a>, ResolveError> {
        let default_resource_id = ResourceId::from(HeapId::from(TypeId::of::<T>()));
        let accessing = resource_id.unwrap_or(&default_resource_id);
        let result = memory_domain.get_shared::<T>(accessing, source)?;

        let access_map = Self::create_and_resolve_access_map(source, Some(accessing.clone()));

        let dropper = DeAccessResolver::new(Arc::clone(memory_domain), access_map);
        let shared = Shared::new(result, dropper);

        Ok(shared)
    }
}

