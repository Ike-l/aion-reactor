use std::sync::Arc;

use crate::prelude::{AccessDropper, AccessMap, DeAccessResolver, Injection, MemoryDomain, MemoryTarget, ReservationAccessMap, ResolveError, ResourceId, SystemId};

#[derive(small_derive_deref::Deref, small_derive_deref::DerefMut)]
pub struct Resulting<'a, T: Injection> {
    #[DerefTarget]
    #[DerefMutTarget]
    inner: Result<T::Item<'a>, ResolveError>,
    access_dropper: Option<DeAccessResolver>
}

impl<'a, T: Injection> Resulting<'a, T> {
    pub fn new(inner: Result<T::Item<'a>, ResolveError>) -> Self {
        let access_dropper = if inner.is_err() {
            Some(DeAccessResolver::new(Arc::new(MemoryDomain::new()), AccessMap::Heap(ReservationAccessMap::default())))
        } else {
            None
        };

        Self {
            inner,
            access_dropper
        }
    }
}

impl<T: Injection> AccessDropper for Resulting<'_, T> {
    fn access_dropper(&self) -> &DeAccessResolver {
        if let Ok(ref inner) = self.inner {
            inner.access_dropper()
        } else {
            self.access_dropper.as_ref().unwrap()
        }
    }
}

impl<T: Injection> Injection for Resulting<'_, T> {
    type Item<'new> = Resulting<'new, T>;

    fn select_memory_target() -> MemoryTarget {
        T::select_memory_target()
    }

    fn create_access_map() -> AccessMap {
        T::create_access_map()
    }

    fn resolve_accesses(access_map: &mut AccessMap, system_id: Option<&SystemId>, resource_id: Option<ResourceId>) {
        T::resolve_accesses(access_map, system_id, resource_id);
    }

    fn failed_message() -> String {
        unreachable!("Cannot fail because will return Err")
    }

    fn retrieve<'a>(memory_domain: &'a Arc<MemoryDomain>, resource_id: Option<&ResourceId>, system_id: Option<&SystemId>) -> Result<Self::Item<'a>, ResolveError> {
        Ok(Resulting::new(T::retrieve(memory_domain, resource_id, system_id)))
    }
}

#[cfg(test)]
mod resulting_tests {
    use std::sync::Arc;

    use crate::prelude::{Resource, Shared};

    use super::*;

    #[test]
    fn resolve_resulting_ok_no_res() {
        let memory_domain = Arc::new(MemoryDomain::new());
        assert!(memory_domain.resolve::<Resulting<Shared<i32>>>(None, None).unwrap().is_err())
    }

    #[test]
    fn resolve_resulting_shared() {
        let memory_domain = Arc::new(MemoryDomain::new());
        
        assert!(memory_domain.insert(ResourceId::from_raw_heap::<i32>(), Resource::dummy(1)).unwrap().is_none());

        assert_eq!(***memory_domain.resolve::<Resulting<Shared<i32>>>(None, None).unwrap().as_ref().unwrap(), 1 as i32);
    }
}