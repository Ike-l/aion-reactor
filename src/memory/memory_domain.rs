use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::{injection::{AccessDropper, injection_trait::Injection}, memory::{ResourceId, access_checked_heap::{AccessCheckedHeap, raw_access_map::RawAccessMap}, access_map::{Access, AccessMap}, errors::{DeResolveError, InsertError, ReservationError, ResolveError}, resource_id::Resource}, system::system_metadata::Source};

// Should be no public way of creating one of these to enforce dropping behaviour by injection types // doesnt matter because the UB would just panic
#[derive(Debug)]
pub struct MemoryDomain {
    heap: AccessCheckedHeap,

    delays: Mutex<HashMap<u64, HashMap<ResourceId, Access>>>
}

impl MemoryDomain {
    pub fn new() -> Self {
        Self {
            heap: AccessCheckedHeap::default(),
            delays: Mutex::new(HashMap::new())
        }
    }

    pub fn ok_resource(&self, resource_id: &ResourceId) -> bool {
        match resource_id {
            ResourceId::Heap(heap_id) => self.heap.ok_resource(&heap_id)
        }
    }

    pub fn ok_access(&self, resource_id: &ResourceId, access: &Access, source: Option<&Source>) -> bool {
        match resource_id {
            ResourceId::Heap(heap_id) => self.heap.ok_access(&heap_id, access, source)
        }
    }

    /// will drain the access map
    pub fn reserve_accesses(&self, source: Source, access_map: AccessMap) -> Result<(), ReservationError> {
        match access_map {
            AccessMap::Heap(access_map) => self.heap.reserve_accesses(&self, source, &mut RawAccessMap::from(access_map))
        }
    }

    pub fn insert(&self, resource_id: ResourceId, resource: Resource) -> Result<Option<Resource>, InsertError> {
        match (resource, resource_id) {
            (Resource::Heap(obj), ResourceId::Heap(id)) => {
                let resource = self.heap.insert(id, obj)?;
                if resource.is_none() {
                    return Ok(None);
                }

                Ok(Some(Resource::Heap(
                    resource.unwrap()
                )))
            }
        }
    }

    pub fn resolve<T: Injection>(self: &Arc<Self>, resource_id: Option<&ResourceId>, source: Option<&Source>) -> Result<T::Item<'_>, ResolveError> {
        let r = T::retrieve(&self, resource_id, source);
        if let Ok(r) = &r {
            // make sure no panics so there MUST be a dropper
            std::hint::black_box(r.access_dropper());
        }

        r
    }

    /// Only to be accessed by the dropper!
    pub(crate) fn delay_drop(&self, accesses: HashMap<ResourceId, Access>) -> u64 {
        let key = rand::random();
        self.delays.lock().unwrap().insert(key, accesses);
        key
    }

    /// Safety:
    /// Do not deaccess something unless you actually free the access!
    pub unsafe fn end_drop_delay(&self, key: &u64) {
        if let Some(accesses) = self.delays.lock().unwrap().remove(key) {
            for (resource_id, access) in accesses {
                unsafe { self.deresolve(access, &resource_id).unwrap() };
            }
        } else {
            panic!("tried to end the drop delay without permission")
        }
    }

    // pub crate for now since i only want the dropper to use this
    /// Safety:
    /// Do not deaccess something unless you actually free the access!
    pub(crate) unsafe fn deresolve(&self, access: Access, resource_id: &ResourceId) -> Result<(), DeResolveError> {
        match resource_id {
            ResourceId::Heap(id) => unsafe { self.heap.deaccess(access, id) }
        }
    }

    pub fn get_cloned<T: 'static + Clone>(&self, resource_id: &ResourceId) -> Result<T, ResolveError> {
        match resource_id {
            ResourceId::Heap(id) => self.heap.get_cloned(id)
        }
    }

    pub fn get_shared<T: 'static>(&self, resource_id: &ResourceId, source: Option<&Source>) -> Result<&T, ResolveError> {
        match resource_id {
            ResourceId::Heap(id) => self.heap.get_shared(id, source)
        }
    }

    pub fn get_unique<T: 'static>(&self, resource_id: &ResourceId, source: Option<&Source>) -> Result<&mut T, ResolveError> {
        match resource_id {
            ResourceId::Heap(id) => self.heap.get_unique(id, source)
        }
    }
}

#[cfg(test)]
mod memory_domain_tests {
    use crate::{id::Id, memory::{ResourceId, access_checked_heap::{heap::HeapId, reservation_access_map::ReservationAccessMap}, access_map::{Access, AccessMap}, errors::ReservationError, memory_domain::MemoryDomain, resource_id::Resource}, system::system_metadata::Source};

    #[test]
    fn delay_end_drop() {
        // test you can delay a drop, then end the drop
        todo!()
    }

    #[test]
    fn reserve_access() {
        let memory_domain = MemoryDomain::new();
        let source = Source(Id("foo".to_string()));
        let mut access_map = ReservationAccessMap::default();

        assert_eq!(memory_domain.reserve_accesses(source.clone(), AccessMap::Heap(access_map.clone())), Ok(()));

        let heap_id1 = HeapId::Label(Id("baz".to_string()));
        assert!(access_map.do_access(heap_id1.clone(), None, Access::Unique).is_ok());
        // memory_domain.get_shared(&ResourceId::Heap(heap_id1.clone()), Some(&source)
        assert_eq!(memory_domain.reserve_accesses(source.clone(), AccessMap::Heap(access_map.clone())), Err(ReservationError::ErrResource));

        assert!(memory_domain.insert(ResourceId::Heap(heap_id1.clone()), Resource::dummy(123)).is_ok());
        assert!(memory_domain.ok_resource(&ResourceId::Heap(heap_id1.clone())));

        assert!(memory_domain.reserve_accesses(source.clone(), AccessMap::Heap(access_map.clone())).is_ok());

        assert!(memory_domain.reserve_accesses(Source(Id("bar".to_string())), AccessMap::Heap(access_map)).is_err());

        assert!(memory_domain.get_shared::<i32>(&ResourceId::Heap(heap_id1.clone()), Some(&source)).is_err());

        assert_eq!(memory_domain.get_cloned::<i32>(&ResourceId::Heap(heap_id1.clone())), Ok(123));

        assert!(memory_domain.get_unique::<i32>(&ResourceId::Heap(heap_id1.clone()), None).is_err());

        assert_eq!(memory_domain.get_unique::<i32>(&ResourceId::Heap(heap_id1.clone()), Some(&source)), Ok(&mut 123));
        
        assert!(unsafe { memory_domain.deresolve(Access::Unique, &ResourceId::Heap(heap_id1.clone())) }.is_ok());

        assert!(memory_domain.get_unique::<i32>(&ResourceId::Heap(heap_id1.clone()), None).is_ok());
    }
}