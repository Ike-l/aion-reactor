use std::collections::HashMap;

use crate::{memory::{access_checked_heap::{heap::HeapId, raw_access_map::RawAccessMap}, access_map::Access, errors::DeResolveError, memory_domain::MemoryDomain}, system::system_metadata::Source};

#[derive(Debug, Default, Clone)]
pub struct ReserveAccessMap {
    access_maps: HashMap<Source, RawAccessMap>
}

impl ReserveAccessMap {
    pub fn is_conflicting_reservation(&self, item: &HeapId, access: &Access, source: Option<&Source>) -> bool {
        for (reserver, access_map) in self.access_maps.iter() {
            if !access_map.ok_access(item, access) {
                if source.map_or(true, |s| s != reserver) {
                    return true;
                } else {
                    if let Some(item) = access_map.get_access(item) {
                        if item.is_semantically_different(access) {
                            return true;
                        }
                    }
                }
            }
        }
        
        return false
    }

    pub fn has_conflicting_reservation(&self, raw_access_map: &RawAccessMap, source: Option<&Source>) -> bool {
        raw_access_map.iter().any(|(item, access)| self.is_conflicting_reservation(item, access, source))
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
    use crate::{id::Id, memory::{access_checked_heap::{heap::HeapId, raw_access_map::RawAccessMap, reserve_access_map::ReserveAccessMap}, access_map::Access, memory_domain::MemoryDomain}, system::system_metadata::Source};

    #[test]
    fn ok_accesses() {
        let reserve_access_map = ReserveAccessMap::default();

        let memory_domain = MemoryDomain::new();
        let source = None;

        assert!(reserve_access_map.ok_accesses(&memory_domain, source));
    }

    #[test]
    fn is_conflicting_reservation() {
        let mut reserve_access_map = ReserveAccessMap::default();

        let item = HeapId::Label(Id("foo".to_string()));
        let access = Access::Shared(1);
        let source = None;
        assert!(!reserve_access_map.is_conflicting_reservation(&item, &access, source));

        let source1 = Source(Id("bar".to_string()));
        let access_map = vec![(
            item.clone(), access.clone()
        )].into_iter();
        
        reserve_access_map.reserve(source1.clone(), access_map);


        assert!(!reserve_access_map.is_conflicting_reservation(&item, &access, source));
        assert!(!reserve_access_map.is_conflicting_reservation(&item, &access, Some(&source1)));
        assert!(reserve_access_map.unreserve(&source1, &item, access.clone()).unwrap().is_ok());

        let access_map = vec![(
            item.clone(), Access::Unique
        )].into_iter();
        
        reserve_access_map.reserve(source1.clone(), access_map);

        assert!(reserve_access_map.is_conflicting_reservation(&item, &access, source));
        assert!(reserve_access_map.is_conflicting_reservation(&item, &access, Some(&source1)));
        assert!(!reserve_access_map.is_conflicting_reservation(&item, &Access::Unique, Some(&source1)));
        assert!(reserve_access_map.unreserve(&source1, &item, Access::Unique).unwrap().is_ok());
        assert!(!reserve_access_map.is_conflicting_reservation(&item, &access, source));
    }

    #[test]
    fn has_conflicting_reservation() {
        let mut reserve_access_map = ReserveAccessMap::default();

        let mut raw_access_map = RawAccessMap::default();
        let source = None;
        assert!(!reserve_access_map.has_conflicting_reservation(&raw_access_map, source));

        let item = HeapId::Label(Id("foo".to_string()));
        let access = Access::Unique;

        assert!(raw_access_map.do_access(item.clone(), access.clone()).is_ok());
        assert!(!reserve_access_map.has_conflicting_reservation(&raw_access_map, source));
        reserve_access_map.reserve(Source(Id("bax".to_string())), raw_access_map.clone().drain());
        assert!(reserve_access_map.has_conflicting_reservation(&raw_access_map, source));
        assert!(reserve_access_map.unreserve(&Source(Id("bax".to_string())), &item, access).unwrap().is_ok());
        reserve_access_map.reserve(Source(Id("bax".to_string())), raw_access_map.clone().drain());
        assert!(!reserve_access_map.has_conflicting_reservation(&raw_access_map, Some(&Source(Id("bax".to_string())))));
    }
}