use crate::{memory::{access_checked_heap::{heap::HeapId, raw_access_map::RawAccessMap, reserve_access_map::ReserveAccessMap}, access_map::Access, errors::{DeResolveError, ResolveError}, memory_domain::MemoryDomain, ResourceId}, system::system_metadata::Source};


#[derive(Debug, Default)]
pub struct HeapAccessMap {
    access_map: RawAccessMap,
    reserve_map: ReserveAccessMap
}

impl HeapAccessMap {
    pub fn drain(&mut self) -> impl Iterator<Item = (HeapId, Access)> {
        self.access_map.drain()
    }

    pub fn ok_resources(&self, memory_domain: &MemoryDomain) -> bool {
        self.access_map.ok_resources(memory_domain)
    }

    pub fn ok_accesses(&self, memory_domain: &MemoryDomain) -> bool {
        self.access_map.ok_accesses(memory_domain) || self.reserve_map.ok_accesses(memory_domain)
    }

    pub fn ok_access(&self, testing_heap_id: &HeapId, testing_access: &Access) -> bool {
        self.access_map.ok_access(testing_heap_id, testing_access) || self.reserve_map.ok_access(testing_heap_id, testing_access)
    }

    pub fn reserve_accesses(&mut self, memory_domain: &MemoryDomain, source: Source, access_map: Self) -> bool {
        if self.ok_accesses(memory_domain) {
            self.reserve_map.reserve(source, access_map.access_map);
            return true;
        }

        false
    }

    pub fn conflicts(&self, other: &Self) -> bool {
        other.access_map.conflicts(&self.access_map)
    }

    pub fn deaccess(&mut self, access: &Access, heap_id: &HeapId) -> Result<(), DeResolveError> {
        self.access_map.deaccess(access, heap_id)
    }

    pub fn get_access(&self, resource_id: &HeapId) -> Option<&Access> {
        self.access_map.get_access(resource_id)
    }

    pub fn access_shared(&mut self, heap_id: HeapId, source: Option<&Source>) -> Result<(), ResolveError> {
        if self.reserve_map.is_conflicting_reservation(&heap_id, &Access::Shared(1), source) {
            return Err(ResolveError::ConflictingReservation(ResourceId::Heap(heap_id)));
        }   

        let result = self.access_map.access_shared(heap_id.clone());
        if let Some(source) = source {
            if result.is_ok() {
                self.reserve_map.unreserve(source, &heap_id, &Access::Shared(1));
            }
        }

        result
    }

    pub fn access_unique(&mut self, heap_id: HeapId, source: Option<&Source>) -> Result<(), ResolveError> {
        
        if self.reserve_map.is_conflicting_reservation(&heap_id, &Access::Unique, source) {
            return Err(ResolveError::ConflictingReservation(ResourceId::Heap(heap_id)));
        }   

        let result = self.access_map.access_unique(heap_id.clone());
        if let Some(source) = source {
            if result.is_ok() {
                self.reserve_map.unreserve(source, &heap_id, &Access::Unique);
            }
        }

        result
    }
}