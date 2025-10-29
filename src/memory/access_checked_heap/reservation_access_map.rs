use crate::{memory::{access_checked_heap::{heap::HeapId, raw_access_map::RawAccessMap, reserve_access_map::ReserveAccessMap}, access_map::Access, errors::{DeResolveError, ResolveError}, memory_domain::MemoryDomain, ResourceId}, system::system_metadata::Source};


#[derive(Debug, Default)]
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
        self.access_map.ok_access(testing_heap_id, testing_access) || self.reserve_map.is_reserved(testing_heap_id, testing_access, source)
    }

    /// will drain the access map
    pub fn reserve_accesses(&mut self, memory_domain: &MemoryDomain, source: Source, access_map: &mut RawAccessMap) -> bool {
        if access_map.ok_accesses(memory_domain, Some(&source)) {
            self.reserve_map.reserve(source, access_map.drain());
            return true;
        }

        false
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
    use crate::{id::Id, memory::{ResourceId, access_checked_heap::{heap::HeapId, raw_access_map::RawAccessMap, reservation_access_map::ReservationAccessMap}, access_map::Access, memory_domain::MemoryDomain, resource_id::Resource}, system::system_metadata::Source};

    #[test]
    fn ok_access() {
        let mut reservation_access_map = ReservationAccessMap::default();

        let testing_heap_id = &HeapId::Label(Id("foo".to_string()));
        let testing_access = &Access::Unique;
        let source = None;

        assert!(reservation_access_map.ok_access(testing_heap_id, testing_access, source));

        assert!(reservation_access_map.do_access(testing_heap_id.clone(), source, Access::Unique).is_ok());

        assert!(!reservation_access_map.ok_access(testing_heap_id, testing_access, source));
    }

    #[test]
    fn ok_accesses() {
        let mut reservation_access_map = ReservationAccessMap::default();
        
        let memory_domain = MemoryDomain::new();
        let source = None;

        assert!(reservation_access_map.ok_accesses(&memory_domain, source));

        let heap_id = HeapId::Label(Id("foo".to_string()));

        assert!(reservation_access_map.do_access(heap_id.clone(), source, Access::Shared(1)).is_ok());
        assert!(!reservation_access_map.ok_accesses(&memory_domain, source));
        assert!(memory_domain.insert(ResourceId::Heap(heap_id), Resource::dummy(123)).is_none());
        assert!(reservation_access_map.ok_accesses(&memory_domain, source));

        todo!("Better testing")
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

        assert!(reservation_access_map.reserve_accesses(&memory_domain, source1.clone(), &mut access_map));

        let heap_id = HeapId::Label(Id("baz".to_string()));
        assert!(access_map.do_access(heap_id.clone(), Access::Shared(1)).is_ok());

        assert!(!reservation_access_map.reserve_accesses(&memory_domain, source1.clone(), &mut access_map));

        memory_domain.insert(ResourceId::Heap(heap_id), Resource::dummy(123));

        assert!(reservation_access_map.reserve_accesses(&memory_domain, source1.clone(), &mut access_map));

        // fails with no res
        let source2 = Source(Id("bar".to_string()));

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