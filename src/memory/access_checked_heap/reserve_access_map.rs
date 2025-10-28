use std::collections::HashMap;

use crate::{memory::{access_checked_heap::{heap::HeapId, raw_access_map::RawAccessMap}, access_map::Access, errors::DeResolveError, memory_domain::MemoryDomain}, system::system_metadata::Source};

#[derive(Debug, Default)]
pub struct ReserveAccessMap {
    access_maps: HashMap<Source, RawAccessMap>
}

impl ReserveAccessMap {
    pub fn is_conflicting_reservation(&self, item: &HeapId, access: &Access, source: Option<&Source>) -> bool {
        for (reserver, access_map) in self.access_maps.iter() {
            if !access_map.ok_access(item, access) {
                if source.map_or(true, |s| s != reserver) {
                    return true;
                }
            }
        }

        return false
    }

    pub fn ok_accesses(&self, memory_domain: &MemoryDomain, source: Option<&Source>) -> bool {
        !self.access_maps.iter().any(|(_, access_map)| !access_map.ok_accesses(memory_domain, source))
    }

    pub fn ok_access(&self, testing_heap_id: &HeapId, testing_access: &Access) -> bool {
        !self.access_maps.iter().any(|(_, access_map)| !access_map.ok_access(testing_heap_id, testing_access))
    }

    pub fn reserve(&mut self, source: Source, access_map: RawAccessMap) {
        self.access_maps.entry(source).or_default().merge(access_map);
    }

    pub fn unreserve(&mut self, source: &Source, item: &HeapId, access: &Access) -> Option<Result<(), DeResolveError>> {
        Some(self.access_maps.get_mut(source)?.deaccess(access, item))
    }
}