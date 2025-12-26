use crate::prelude::{Access, DeResolveError, HeapId, MemoryDomain, RawAccessMap, ReservationError, ReserveAccessMap, ResolveError, ResourceId, SystemId};


#[derive(Debug, Default, Clone)]
pub struct ReservationAccessMap {
    access_map: RawAccessMap,
    reserve_map: ReserveAccessMap
}

impl ReservationAccessMap {
    pub fn is_read_only(&self) -> bool {
        self.access_map.is_read_only()
    }

    pub fn drain(&mut self) -> impl Iterator<Item = (HeapId, Access)> {
        self.access_map.drain()
    }

    /// are all resources in self (accesses) also in memory_domain
    pub fn ok_resources(&self, memory_domain: &MemoryDomain) -> bool {
        self.access_map.ok_resources(memory_domain)
    }

    pub fn ok_accesses(&self, memory_domain: &MemoryDomain, system_id: Option<&SystemId>) -> bool {
        // if current accesses dont conflict and no reservation conflict with memory
        self.access_map.ok_accesses(memory_domain, system_id) && self.reserve_map.ok_accesses(memory_domain, system_id)
    }

    /// an access is ok if either 1. there is no conflicting access or 2. the access has been reserved
    pub fn ok_access(&self, testing_heap_id: &HeapId, testing_access: &Access, system_id: Option<&SystemId>) -> bool {
        self.access_map.ok_access(testing_heap_id, testing_access) && !self.reserve_map.is_conflicting_reservation(testing_heap_id, testing_access, system_id)
    }

    pub fn unreserve(&mut self, heap_id: &HeapId, access: Access, system_id: &SystemId) {
        self.reserve_map.unreserve(system_id, heap_id, access);
    }

    pub fn ok_reservation_self(&self, other: &Self, system_id: Option<&SystemId>, memory_domain: &MemoryDomain) -> Option<ReservationError> {
        self.ok_reservation(&other.access_map, system_id, memory_domain)
    }

    pub fn ok_reservation(&self, other: &RawAccessMap, system_id: Option<&SystemId>, memory_domain: &MemoryDomain) -> Option<ReservationError> {
        if self.reserve_map.has_conflicting_reservation(&other, system_id) {
            return Some(ReservationError::ConflictingReservation);
        }

        if !other.ok_resources(memory_domain) {
            return Some(ReservationError::ErrResource);
        }

        // May lead to a bug in the future, basically
        // i am making the assumption that if we are checking for an access that access is always only in one place (self),
        // a more abstract version would ask memory domain if there is a conflict however that leads to a deadlock over self (currently).
        if self.access_map.conflicts(&other) {
            return Some(ReservationError::ConcurrentAccess);
        }

        None
    }

    /// will drain the access map
    pub fn reserve_accesses(&mut self, memory_domain: &MemoryDomain, system_id: SystemId, access_map: &mut RawAccessMap) -> Result<(), ReservationError> {
        if let Some(err) = self.ok_reservation(&access_map, Some(&system_id), memory_domain) {
            return Err(err);
        }
        
        self.reserve_map.reserve(system_id, access_map.drain());
        Ok(())
    }

    /// will drain the access map
    pub fn reserve_accesses_self(&mut self, memory_domain: &MemoryDomain, system_id: SystemId, other: &mut Self) -> Result<(), ReservationError> {
        self.reserve_accesses(memory_domain, system_id, &mut other.access_map)
    }

    pub fn reserve_current_accesses(&mut self, system_id: SystemId, access_map: &mut RawAccessMap) -> Result<(), ReservationError> {
        if self.reserve_map.has_conflicting_reservation(&access_map, Some(&system_id)) {
            return Err(ReservationError::ConflictingReservation);
        }

        // May lead to a bug in the future, basically
        // i am making the assumption that if we are checking for an access that access is always only in one place (self),
        // a more abstract version would ask memory domain if there is a conflict however that leads to a deadlock over self (currently).
        if self.access_map.conflicts(&access_map) {
            return Err(ReservationError::ConcurrentAccess);
        }
        
        self.reserve_map.reserve(system_id, access_map.drain());
        Ok(())
    }

    pub fn conflicts(&self, other: &Self) -> bool {
        other.access_map.conflicts(&self.access_map)
    }

    pub fn deaccess(&mut self, access: Access, heap_id: &HeapId) -> Result<(), DeResolveError> {
        self.access_map.deaccess(access, heap_id)
    }

    pub fn get_access(&self, resource_id: &HeapId) -> Option<&Access> {
        self.access_map.get_access(resource_id)
    }

    pub fn do_access(&mut self, heap_id: HeapId, system_id: Option<&SystemId>, access: Access) -> Result<(), ResolveError> {
        if self.reserve_map.is_conflicting_reservation(&heap_id, &access, system_id) {
            return Err(ResolveError::ConflictingReservation(ResourceId::Heap(heap_id)));
        }


        let result = self.access_map.do_access(heap_id.clone(), access.clone());
        if let Some(system_id) = system_id {
            if result.is_ok() {
                self.reserve_map.unreserve(system_id, &heap_id, access);
            }
        }

        result
    }
}

#[cfg(test)]
mod reservation_access_map_tests {
    use crate::prelude::{Access, AccessMap, HeapId, Id, MemoryDomain, ReservationAccessMap, Resource, ResourceId, SystemId};

    #[test]
    fn ok_access() {
        let mut reservation_access_map = ReservationAccessMap::default();

        let testing_heap_id = &HeapId::Label(Id::from("foo"));
        let testing_access = &Access::Unique;
        let system_id = None;

        assert!(reservation_access_map.ok_access(testing_heap_id, testing_access, system_id));

        assert!(reservation_access_map.do_access(testing_heap_id.clone(), system_id, Access::Unique).is_ok());

        assert!(!reservation_access_map.ok_access(testing_heap_id, testing_access, system_id));
        // todo!("if the access has been reserved then fails") // done in MemoryDomain::reserve..
    }

    #[test]
    fn ok_accesses() {
        let mut reservation_access_map = ReservationAccessMap::default();
        
        let memory_domain = MemoryDomain::new();
        let system_id = None;

        assert!(reservation_access_map.ok_accesses(&memory_domain, system_id));

        let heap_id = HeapId::Label(Id::from("foo"));

        // no resource
        assert!(reservation_access_map.do_access(heap_id.clone(), system_id, Access::Shared(1)).is_ok());
        assert!(!reservation_access_map.ok_accesses(&memory_domain, system_id));

        assert!(memory_domain.insert(ResourceId::Heap(heap_id.clone()), Resource::dummy(123)).unwrap().is_none());
        assert!(reservation_access_map.ok_accesses(&memory_domain, system_id));

        // conflict
        assert!(memory_domain.get_unique::<i32>(&ResourceId::Heap(heap_id.clone()), None).is_ok());
        assert!(!reservation_access_map.ok_accesses(&memory_domain, system_id));
        assert!(unsafe { memory_domain.deresolve(Access::Unique, &ResourceId::Heap(heap_id.clone())) }.is_ok());
        assert!(reservation_access_map.ok_accesses(&memory_domain, system_id));
        assert!(memory_domain.get_shared::<i32>(&ResourceId::Heap(heap_id.clone()), None).is_ok());
        assert!(reservation_access_map.ok_accesses(&memory_domain, system_id));
        assert!(unsafe { memory_domain.deresolve(Access::Shared(1), &ResourceId::Heap(heap_id.clone())) }.is_ok());
        
        let system_id = SystemId::from("baz");
        let mut reservation_access_map = ReservationAccessMap::default();
        assert!(reservation_access_map.do_access(heap_id.clone(), None, Access::Unique).is_ok());
        assert!(memory_domain.reserve_accesses(system_id.clone(), AccessMap::Heap(reservation_access_map.clone())).is_ok());
        assert!(reservation_access_map.ok_accesses(&memory_domain, Some(&system_id)));
        assert!(!reservation_access_map.ok_accesses(&memory_domain, None));
    }

    #[test]
    fn access_shared() {
        let mut reservation_access_map = ReservationAccessMap::default();
        
        let heap_id = HeapId::Label(Id::from("foo"));
        let system_id = None;

        assert!(reservation_access_map.do_access(heap_id.clone(), system_id, Access::Shared(1)).is_ok());
        assert!(reservation_access_map.do_access(heap_id, system_id, Access::Shared(1)).is_ok());
    }

    #[test]
    fn access_unique() {
        let mut reservation_access_map = ReservationAccessMap::default();
        
        let heap_id = HeapId::Label(Id::from("foo"));
        let system_id = None;

        assert!(reservation_access_map.do_access(heap_id.clone(), system_id, Access::Unique).is_ok());
        assert!(reservation_access_map.do_access(heap_id, system_id, Access::Unique).is_err());
    }

    #[test]
    fn access_shared_and_unique() {
        let mut reservation_access_map = ReservationAccessMap::default();
        
        let heap_id = HeapId::Label(Id::from("foo"));
        let system_id = None;

        assert!(reservation_access_map.do_access(heap_id.clone(), system_id, Access::Shared(1)).is_ok());
        assert!(reservation_access_map.do_access(heap_id.clone(), system_id, Access::Unique).is_err());
        assert!(reservation_access_map.do_access(heap_id.clone(), system_id, Access::Shared(1)).is_ok());
        assert!(reservation_access_map.deaccess(Access::Shared(2), &heap_id).is_ok());
        assert!(reservation_access_map.do_access(heap_id, system_id, Access::Unique).is_ok());
    }

    /*
    // #[test]
    // // fn reserve() {
    // //     let mut reservation_access_map = ReservationAccessMap::default();

    // //     let memory_domain = MemoryDomain::new();
    // //     let source1 = SystemId(Id("foo".to_string()));
    // //     let mut access_map = RawAccessMap::default();

    // //     assert!(reservation_access_map.reserve_accesses(&memory_domain, source1.clone(), &mut access_map).is_ok());

    // //     let heap_id = HeapId::Label(Id("baz".to_string()));
    // //     assert!(access_map.do_access(heap_id.clone(), Access::Shared(1)).is_ok());

    // //     assert!(reservation_access_map.reserve_accesses(&memory_domain, source1.clone(), &mut access_map).is_err());

    // //     assert!(memory_domain.insert(ResourceId::Heap(heap_id), Resource::dummy(123)).is_ok());

    // //     assert!(reservation_access_map.reserve_accesses(&memory_domain, source1.clone(), &mut access_map).is_ok());

    // //     // fails with no res
    // //     let _source2 = SystemId(Id("bar".to_string()));

    // //     todo!("finish testing");
    // //     // access shared and fails reserve
    // //     // access unique and fails reserve
    // //     // no access succeed reserve(shared) then fails unique then accesses shared
    // //     // no access succeed reserve(shared) then succeeds shared then accesses shared
    // //     // no access succeed reserve(unique) then fails shared then accesses unique
    // //     // no access succeed reserve(unique) then fails shared then fails accesses shared
    // // }
    */

    /*
    #[test]
    fn ok_resourcea() {
        let reservation_access_map = ReservationAccessMap::default();



        reservation_access_map.ok_resources(memory_domain);

        todo!()
    }
    */
    /*
    #[test]
    fn conflicts() {
        todo!()
    }
    */

    /*
    #[test]
    fn deaccess() {
        todo!()
    }
    */
}