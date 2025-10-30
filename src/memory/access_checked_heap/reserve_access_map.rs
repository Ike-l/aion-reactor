use std::collections::HashMap;

use crate::{memory::{access_checked_heap::{heap::HeapId, raw_access_map::RawAccessMap}, access_map::Access, errors::DeResolveError, memory_domain::MemoryDomain}, system::system_metadata::Source};

#[derive(Debug, Default, Clone)]
pub struct ReserveAccessMap {
    access_maps: HashMap<Source, RawAccessMap>
}

impl ReserveAccessMap {
    pub fn is_conflicting_reservation(&self, item: &HeapId, access: &Access, source: Option<&Source>) -> bool {
        for (reserver, access_map) in self.access_maps.iter() {
            // if accesses are not compatible then check source.
            // if accesses are compatible continue
            // however if they are not the same access then  

            // if the accesses conflict
            if !access_map.ok_access(item, access) {
                // and if the reserver is not the same
                if source.map_or(true, |s| s != reserver) {
                    // signal conflict
                    return true;
                } else {
                    // should never fail by the nature of ok_access but if it does not worth panicking over
                    if let Some(item) = access_map.get_access(item) {
                        if item.is_semantically_different(access) {
                            return true;
                        }
                    }
                    // if access and the incompatible access are not the same also return true
                }
            }
        }
        
        return false
    }

    pub fn has_conflicting_reservation(&self, raw_access_map: &RawAccessMap, source: Option<&Source>) -> bool {
        raw_access_map.iter().any(|(item, access)| self.is_conflicting_reservation(item, access, source))
    }

    pub fn is_reserved_by(&self, item: &HeapId, access: &Access, source: &Source) -> bool {
        if let Some(source) = self.access_maps.get(source) {
            return source.get_access(item).is_some_and(|existing_access| !existing_access.is_semantically_different(access))
        }

        false
    }

    /// if any of the reservation maps conflicts with memory
    pub fn ok_accesses(&self, memory_domain: &MemoryDomain, source: Option<&Source>) -> bool {
        !self.access_maps.iter().any(|(_, access_map)| !access_map.ok_accesses(memory_domain, source))
    }

    pub fn ok_access(&self, testing_heap_id: &HeapId, testing_access: &Access) -> bool {
        !self.access_maps.iter().any(|(_, access_map)| !access_map.ok_access(testing_heap_id, testing_access))
    }

    pub fn reserve(&mut self, source: Source, access_map: impl Iterator<Item = (HeapId, Access)>) {
        self.access_maps.entry(source).or_default().merge(access_map);
    }

    pub fn unreserve(&mut self, source: &Source, item: &HeapId, access: Access) -> Option<Result<(), DeResolveError>> {
        Some(self.access_maps.get_mut(source)?.deaccess(access, item))
    }
}

#[cfg(test)]
mod reserve_access_map_tests {
    use crate::memory::{access_checked_heap::reserve_access_map::ReserveAccessMap, memory_domain::MemoryDomain};

    #[test]
    fn ok_accesses() {
        let reserve_access_map = ReserveAccessMap::default();

        let memory_domain = MemoryDomain::new();
        let source = None;

        assert!(reserve_access_map.ok_accesses(&memory_domain, source));
    }

    #[test]
    fn is_reserved() {
        todo!()
    }

    #[test]
    fn is_conflicting_reservation() {
        todo!()
    }

    #[test]
    fn has_conflicting_reservation() {
        todo!()
    }

    #[test]
    fn ok_access() {
        todo!()
    }

    #[test]
    fn reserve_unreserve() {
        todo!()
    }
}