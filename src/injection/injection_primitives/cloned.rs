use std::{any::{type_name, TypeId}, sync::Arc};

use crate::{injection::{injection_trait::Injection, DeAccessResolver, AccessDropper}, memory::{access_checked_heap::{heap::HeapId, reservation_access_map::ReservationAccessMap}, access_map::AccessMap, errors::ResolveError, memory_domain::MemoryDomain, ResourceId}, system::system_metadata::Source};

#[derive(Debug, small_derive_deref::Deref, small_derive_deref::DerefMut)]
pub struct Cloned<T> {
    #[DerefTarget]
    #[DerefMutTarget]
    value: T,
    dropper: DeAccessResolver
}

impl<T: 'static> Cloned<T> {
    pub fn new(value: T, dropper: DeAccessResolver) -> Self {
        Self {
            value,
            dropper
        }
    }
}

impl<T> AccessDropper for Cloned<T> {
    fn access_dropper(&self) -> &DeAccessResolver {
        &self.dropper
    }
}

impl<T: 'static + Clone> Injection for Cloned<T> {
    type Item<'new> = Cloned<T>;

    fn failed_message() -> String {
        format!("Expected Resource: `{}`", type_name::<T>())
    }

    fn create_access_map() -> AccessMap {
        AccessMap::Heap(ReservationAccessMap::default())
    }

    fn resolve_accesses(_access_map: &mut AccessMap, _source: Option<&Source>, _resource_id: Option<ResourceId>) {}

    fn retrieve<'a>(memory_domain: &'a Arc<MemoryDomain>, resource_id: Option<&ResourceId>, source: Option<&Source>) -> Result<Self::Item<'a>, ResolveError> {
        let default_resource_id = ResourceId::from(HeapId::from(TypeId::of::<T>()));
        let accessing = resource_id.unwrap_or(&default_resource_id);
        let result =  memory_domain.get_cloned::<T>(accessing)?;

        let access_map = Self::create_and_resolve_access_map(source, Some(accessing.clone()));

        let dropper = DeAccessResolver::new(Arc::clone(memory_domain), access_map);
        let cloned = Cloned::new(result, dropper);

        Ok(cloned)
    }
}