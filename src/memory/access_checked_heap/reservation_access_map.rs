use crate::{memory::{ResourceId, access_checked_heap::{heap::HeapId, raw_access_map::RawAccessMap, reserve_access_map::ReserveAccessMap}, access_map::Access, errors::{DeResolveError, ReservationError, ResolveError}, memory_domain::MemoryDomain}, system::system_metadata::Source};


#[derive(Debug, Default, Clone)]
pub struct ReservationAccessMap {
    access_map: RawAccessMap,
    reserve_map: ReserveAccessMap
}

impl ReservationAccessMap {
    pub fn drain(&mut self) -> impl Iterator<Item = (HeapId, Access)> {
        self.access_map.drain()
    }

    /// are all resources in self (accesses) also in memory_domain
    pub fn ok_resources(&self, memory_domain: &MemoryDomain) -> bool {
        self.access_map.ok_resources(memory_domain)
    }

    pub fn ok_accesses(&self, memory_domain: &MemoryDomain, source: Option<&Source>) -> bool {
        // if current accesses dont conflict and no reservation conflict with memory
        self.access_map.ok_accesses(memory_domain, source) && self.reserve_map.ok_accesses(memory_domain, source)
    }

    /// an access is ok if either 1. there is no conflicting access or 2. the access has been reserved
    pub fn ok_access(&self, testing_heap_id: &HeapId, testing_access: &Access, source: Option<&Source>) -> bool {
        self.access_map.ok_access(testing_heap_id, testing_access) && !self.reserve_map.is_conflicting_reservation(testing_heap_id, testing_access, source)
    }

    pub fn unreserve(&mut self, heap_id: &HeapId, access: Access, source: &Source) {
        self.reserve_map.unreserve(source, heap_id, access);
    }

    /// will drain the access map
    pub fn reserve_accesses(&mut self, memory_domain: &MemoryDomain, source: Source, access_map: &mut RawAccessMap) -> Result<(), ReservationError> {
        if self.reserve_map.has_conflicting_reservation(&access_map, Some(&source)) {
            return Err(ReservationError::ConflictingReservation);
        }

        if !access_map.ok_resources(memory_domain) {
            return Err(ReservationError::ErrResource);
        }

        // May lead to a bug in the future, basically
        // i am making the assumption that if we are checking for an access that access is always only in one place (self),
        // a more abstract version would ask memory domain if there is a conflict however that leads to a deadlock over self (currently).
        if self.access_map.conflicts(&access_map) {
            return Err(ReservationError::ConcurrentAccess);
        }
        
        self.reserve_map.reserve(source, access_map.drain());
        Ok(())
    }

    pub fn reserve_current_accesses(&mut self, source: Source, access_map: &mut RawAccessMap) -> Result<(), ReservationError> {
        if self.reserve_map.has_conflicting_reservation(&access_map, Some(&source)) {
            return Err(ReservationError::ConflictingReservation);
        }

        // May lead to a bug in the future, basically
        // i am making the assumption that if we are checking for an access that access is always only in one place (self),
        // a more abstract version would ask memory domain if there is a conflict however that leads to a deadlock over self (currently).
        if self.access_map.conflicts(&access_map) {
            return Err(ReservationError::ConcurrentAccess);
        }
        
        self.reserve_map.reserve(source, access_map.drain());
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

    pub fn do_access(&mut self, heap_id: HeapId, source: Option<&Source>, access: Access) -> Result<(), ResolveError> {
        if self.reserve_map.is_conflicting_reservation(&heap_id, &access, source) {
            return Err(ResolveError::ConflictingReservation(ResourceId::Heap(heap_id)));
        }


        let result = self.access_map.do_access(heap_id.clone(), access.clone());
        if let Some(source) = source {
            if result.is_ok() {
                self.reserve_map.unreserve(source, &heap_id, access);
            }
        }

        result
    }
}

#[cfg(test)]
mod reservation_access_map_tests {
    use crate::{id::Id, memory::{ResourceId, access_checked_heap::{heap::HeapId, raw_access_map::RawAccessMap, reservation_access_map::ReservationAccessMap}, access_map::{Access, AccessMap}, memory_domain::MemoryDomain, resource_id::Resource}, system::system_metadata::Source};

    #[test]
    fn ok_access() {
        let mut reservation_access_map = ReservationAccessMap::default();

        let testing_heap_id = &HeapId::Label(Id("foo".to_string()));
        let testing_access = &Access::Unique;
        let source = None;

        assert!(reservation_access_map.ok_access(testing_heap_id, testing_access, source));

        assert!(reservation_access_map.do_access(testing_heap_id.clone(), source, Access::Unique).is_ok());

        assert!(!reservation_access_map.ok_access(testing_heap_id, testing_access, source));
        // todo!("if the access has been reserved then fails") // done in MemoryDomain::reserve..
    }

    #[test]
    fn ok_accesses() {
        let mut reservation_access_map = ReservationAccessMap::default();
        
        let memory_domain = MemoryDomain::new();
        let source = None;

        assert!(reservation_access_map.ok_accesses(&memory_domain, source));

        let heap_id = HeapId::Label(Id("foo".to_string()));

        // no resource
        assert!(reservation_access_map.do_access(heap_id.clone(), source, Access::Shared(1)).is_ok());
        assert!(!reservation_access_map.ok_accesses(&memory_domain, source));

        assert!(memory_domain.insert(ResourceId::Heap(heap_id.clone()), Resource::dummy(123)).unwrap().is_none());
        assert!(reservation_access_map.ok_accesses(&memory_domain, source));

        // conflict
        assert!(memory_domain.get_unique::<i32>(&ResourceId::Heap(heap_id.clone()), None).is_ok());
        assert!(!reservation_access_map.ok_accesses(&memory_domain, source));
        assert!(unsafe { memory_domain.deresolve(Access::Unique, &ResourceId::Heap(heap_id.clone())) }.is_ok());
        assert!(reservation_access_map.ok_accesses(&memory_domain, source));
        assert!(memory_domain.get_shared::<i32>(&ResourceId::Heap(heap_id.clone()), None).is_ok());
        assert!(reservation_access_map.ok_accesses(&memory_domain, source));
        assert!(unsafe { memory_domain.deresolve(Access::Shared(1), &ResourceId::Heap(heap_id.clone())) }.is_ok());
        
        let source = Source(Id("baz".to_string()));
        let mut reservation_access_map = ReservationAccessMap::default();
        assert!(reservation_access_map.do_access(heap_id.clone(), None, Access::Unique).is_ok());
        assert!(memory_domain.reserve_accesses(source.clone(), AccessMap::Heap(reservation_access_map.clone())).is_ok());
        assert!(reservation_access_map.ok_accesses(&memory_domain, Some(&source)));
        assert!(!reservation_access_map.ok_accesses(&memory_domain, None));
    }

    #[test]
    fn access_shared() {
        let mut reservation_access_map = ReservationAccessMap::default();
        
        let heap_id = HeapId::Label(Id("foo".to_string()));
        let source = None;

        assert!(reservation_access_map.do_access(heap_id.clone(), source, Access::Shared(1)).is_ok());
        assert!(reservation_access_map.do_access(heap_id, source, Access::Shared(1)).is_ok());
    }

    #[test]
    fn access_unique() {
        let mut reservation_access_map = ReservationAccessMap::default();
        
        let heap_id = HeapId::Label(Id("foo".to_string()));
        let source = None;

        assert!(reservation_access_map.do_access(heap_id.clone(), source, Access::Unique).is_ok());
        assert!(reservation_access_map.do_access(heap_id, source, Access::Unique).is_err());
    }

    #[test]
    fn access_shared_and_unique() {
        let mut reservation_access_map = ReservationAccessMap::default();
        
        let heap_id = HeapId::Label(Id("foo".to_string()));
        let source = None;

        assert!(reservation_access_map.do_access(heap_id.clone(), source, Access::Shared(1)).is_ok());
        assert!(reservation_access_map.do_access(heap_id.clone(), source, Access::Unique).is_err());
        assert!(reservation_access_map.do_access(heap_id.clone(), source, Access::Shared(1)).is_ok());
        assert!(reservation_access_map.deaccess(Access::Shared(2), &heap_id).is_ok());
        assert!(reservation_access_map.do_access(heap_id, source, Access::Unique).is_ok());
    }

    #[test]
    fn reserve() {
        let mut reservation_access_map = ReservationAccessMap::default();

        let memory_domain = MemoryDomain::new();
        let source1 = Source(Id("foo".to_string()));
        let mut access_map = RawAccessMap::default();

        assert!(reservation_access_map.reserve_accesses(&memory_domain, source1.clone(), &mut access_map).is_ok());

        let heap_id = HeapId::Label(Id("baz".to_string()));
        assert!(access_map.do_access(heap_id.clone(), Access::Shared(1)).is_ok());

        assert!(reservation_access_map.reserve_accesses(&memory_domain, source1.clone(), &mut access_map).is_err());

        assert!(memory_domain.insert(ResourceId::Heap(heap_id), Resource::dummy(123)).is_ok());

        assert!(reservation_access_map.reserve_accesses(&memory_domain, source1.clone(), &mut access_map).is_ok());

        // fails with no res
        let _source2 = Source(Id("bar".to_string()));

        todo!("finish testing");
        // access shared and fails reserve
        // access unique and fails reserve
        // no access succeed reserve(shared) then fails unique then accesses shared
        // no access succeed reserve(shared) then succeeds shared then accesses shared
        // no access succeed reserve(unique) then fails shared then accesses unique
        // no access succeed reserve(unique) then fails shared then fails accesses shared
    }

    #[test]
    fn ok_resource() {
        todo!()
    }

    #[test]
    fn conflicts() {
        todo!()
    }

    #[test]
    fn deaccess() {
        todo!()
    }
}