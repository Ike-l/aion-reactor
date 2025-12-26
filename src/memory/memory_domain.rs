use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::prelude::{Access, AccessCheckedHeap, AccessDropper, AccessMap, DeResolveError, Injection, InsertError, RawAccessMap, ReservationError, ResolveError, Resource, ResourceId, SystemId};

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

    pub fn ok_access(&self, resource_id: &ResourceId, access: &Access, system_id: Option<&SystemId>) -> bool {
        match resource_id {
            ResourceId::Heap(heap_id) => self.heap.ok_access(&heap_id, access, system_id)
        }
    }

    /// will drain the access map
    pub fn reserve_accesses(&self, system_id: SystemId, access_map: AccessMap) -> Result<(), ReservationError> {
        match access_map {
            AccessMap::Heap(access_map) => self.heap.reserve_accesses(&self, system_id, &mut RawAccessMap::from(access_map))
        }
    }

    /// will drain the other memory
    pub fn reserve_accesses_self(&self, system_id: SystemId, mut other: Self) -> Result<(), ReservationError> {
        self.heap.reserve_accesses_self(&self, system_id, &mut other.heap)
    }

    pub fn reserve_current_accesses(&self, system_id: SystemId, access_map: AccessMap) -> Result<(), ReservationError> {
        match access_map {
            AccessMap::Heap(access_map) => self.heap.reserve_current_accesses(system_id, &mut RawAccessMap::from(access_map))
        }
    }

    pub fn ok_reservation_self(&self, other: &Self, system_id: Option<&SystemId>) -> Option<ReservationError> {
        self.heap.ok_reservation_self(&other.heap, system_id, &self)
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

    pub fn resolve<T: Injection>(self: &Arc<Self>, resource_id: Option<&ResourceId>, system_id: Option<&SystemId>) -> Result<T::Item<'_>, ResolveError> {
        let r = T::retrieve(&self, resource_id, system_id);
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

    pub fn get_shared<T: 'static>(&self, resource_id: &ResourceId, system_id: Option<&SystemId>) -> Result<&T, ResolveError> {
        match resource_id {
            ResourceId::Heap(id) => self.heap.get_shared(id, system_id)
        }
    }

    pub fn get_unique<T: 'static>(&self, resource_id: &ResourceId, system_id: Option<&SystemId>) -> Result<&mut T, ResolveError> {
        match resource_id {
            ResourceId::Heap(id) => self.heap.get_unique(id, system_id)
        }
    }
}

#[cfg(test)]
mod memory_domain_tests {
    use crate::prelude::{Access, AccessMap, HeapId, Id, MemoryDomain, ReservationAccessMap, ReservationError, Resource, ResourceId, SystemId};

    #[test]
    fn reserve_access() {
        let memory_domain = MemoryDomain::new();
        let system_id = SystemId::from("foo");
        let mut access_map = ReservationAccessMap::default();

        assert_eq!(memory_domain.reserve_accesses(system_id.clone(), AccessMap::Heap(access_map.clone())), Ok(()));

        let heap_id1 = HeapId::Label(Id::from("baz"));
        assert!(access_map.do_access(heap_id1.clone(), None, Access::Unique).is_ok());
        // memory_domain.get_shared(&ResourceId::Heap(heap_id1.clone()), Some(&system_id)
        assert_eq!(memory_domain.reserve_accesses(system_id.clone(), AccessMap::Heap(access_map.clone())), Err(ReservationError::ErrResource));

        assert!(memory_domain.insert(ResourceId::Heap(heap_id1.clone()), Resource::dummy(123)).is_ok());
        assert!(memory_domain.ok_resource(&ResourceId::Heap(heap_id1.clone())));

        assert!(memory_domain.reserve_accesses(system_id.clone(), AccessMap::Heap(access_map.clone())).is_ok());

        assert!(memory_domain.reserve_accesses(SystemId::from("bar"), AccessMap::Heap(access_map.clone())).is_err());

        assert!(memory_domain.get_shared::<i32>(&ResourceId::Heap(heap_id1.clone()), Some(&system_id)).is_err());

        assert_eq!(memory_domain.get_cloned::<i32>(&ResourceId::Heap(heap_id1.clone())), Ok(123));

        assert!(memory_domain.get_unique::<i32>(&ResourceId::Heap(heap_id1.clone()), None).is_err());

        assert_eq!(memory_domain.get_unique::<i32>(&ResourceId::Heap(heap_id1.clone()), Some(&system_id)), Ok(&mut 123));
        
        assert!(unsafe { memory_domain.deresolve(Access::Unique, &ResourceId::Heap(heap_id1.clone())) }.is_ok());

        assert!(memory_domain.get_unique::<i32>(&ResourceId::Heap(heap_id1.clone()), None).is_ok());

        assert!(memory_domain.reserve_accesses(system_id, AccessMap::Heap(access_map)).is_err())
    }
}