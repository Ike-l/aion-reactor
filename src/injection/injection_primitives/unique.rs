use std::{any::{TypeId, type_name}, fmt::{Debug, Display}, sync::Arc};

use crate::prelude::{Access, AccessDropper, AccessMap, DeAccessResolver, HeapId, Injection, MemoryDomain, ReservationAccessMap, ResolveError, ResourceId, SystemId};

#[derive(small_derive_deref::Deref, small_derive_deref::DerefMut)]
pub struct Unique<'a, T> {
    #[DerefTarget]
    #[DerefMutTarget]
    value: &'a mut T,
    dropper: DeAccessResolver
}

impl<T> Debug for Unique<'_, T> 
    where T: Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }   
}

impl<T> Display for Unique<'_, T> 
    where T: Display
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }   
}

impl<'a, T: 'static> Unique<'a, T> {
    pub fn new(value: &'a mut T, dropper: DeAccessResolver) -> Self {
        Self {
            value,
            dropper
        }
    }
}

impl<T> AccessDropper for Unique<'_, T> {
    fn access_dropper(&self) -> &DeAccessResolver {
        &self.dropper
    }
}

impl<T: 'static> Injection for Unique<'_, T> {
    type Item<'new> = Unique<'new, T>;

    fn failed_message() -> String {
        format!("Expected Resource: `{}`", type_name::<T>())
    }

    fn create_access_map() -> AccessMap {
        AccessMap::Heap(ReservationAccessMap::default())
    }

    fn resolve_accesses(access_map: &mut AccessMap, system_id: Option<&SystemId>, resource_id: Option<ResourceId>) {
        match (access_map, resource_id.unwrap_or(ResourceId::Heap(HeapId::RawType(TypeId::of::<T>())))) {
            (AccessMap::Heap(access_map), ResourceId::Heap(heap_id)) => access_map.do_access(heap_id, system_id, Access::Unique).unwrap()
        }
    }

    fn retrieve<'a>(memory_domain: &'a Arc<MemoryDomain>, resource_id: Option<&ResourceId>, system_id: Option<&SystemId>) -> Result<Self::Item<'a>, ResolveError> {
        let default_resource_id = ResourceId::from_raw_heap::<T>();
        let accessing = resource_id.clone().unwrap_or(&default_resource_id);
        let result = memory_domain.get_unique::<T>(accessing, system_id)?;

        let mut access_map = Self::create_access_map();
        Self::resolve_accesses(&mut access_map, system_id, Some(accessing.clone()));
        // let access_map = Self::create_and_resolve_access_map(system_id, Some(accessing.clone()));

        let dropper = DeAccessResolver::new(Arc::clone(memory_domain), access_map);
        let shared = Unique::new(result, dropper);

        Ok(shared)
    }
}