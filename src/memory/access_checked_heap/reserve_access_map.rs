use std::collections::HashMap;

use crate::{ids::system_id::SystemId, memory::{access_checked_heap::{heap::HeapId, raw_access_map::RawAccessMap}, access_map::Access, errors::DeResolveError, memory_domain::MemoryDomain} };

#[derive(Debug, Default, Clone)]
pub struct ReserveAccessMap {
    access_maps: HashMap<SystemId, RawAccessMap>
}

impl ReserveAccessMap {
    pub fn is_conflicting_reservation(&self, item: &HeapId, access: &Access, system_id: Option<&SystemId>) -> bool {
        for (reserver, access_map) in self.access_maps.iter() {
            if !access_map.ok_access(item, access) {
                if system_id.map_or(true, |s| s != reserver) {
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

    pub fn has_conflicting_reservation(&self, raw_access_map: &RawAccessMap, system_id: Option<&SystemId>) -> bool {
        raw_access_map.iter().any(|(item, access)| self.is_conflicting_reservation(item, access, system_id))
    }

    /// if any of the reservation maps conflicts with memory
    pub fn ok_accesses(&self, memory_domain: &MemoryDomain, system_id: Option<&SystemId>) -> bool {
        !self.access_maps.iter().any(|(_, access_map)| !access_map.ok_accesses(memory_domain, system_id))
    }

    pub fn ok_access(&self, testing_heap_id: &HeapId, testing_access: &Access) -> bool {
        !self.access_maps.iter().any(|(_, access_map)| !access_map.ok_access(testing_heap_id, testing_access))
    }

    pub fn reserve(&mut self, system_id: SystemId, access_map: impl Iterator<Item = (HeapId, Access)>) {
        self.access_maps.entry(system_id).or_default().merge(access_map);
    }

    pub fn unreserve(&mut self, system_id: &SystemId, item: &HeapId, access: Access) -> Option<Result<(), DeResolveError>> {
        Some(self.access_maps.get_mut(system_id)?.deaccess(access, item))
    }
}

#[cfg(test)]
mod reserve_access_map_tests {
    use crate::prelude::{Access, HeapId, Id, MemoryDomain, RawAccessMap, ReserveAccessMap, SystemId};

    #[test]
    fn ok_accesses() {
        let reserve_access_map = ReserveAccessMap::default();

        let memory_domain = MemoryDomain::new();
        let system_id = None;

        assert!(reserve_access_map.ok_accesses(&memory_domain, system_id));
    }

    #[test]
    fn is_conflicting_reservation() {
        let mut reserve_access_map = ReserveAccessMap::default();

        let item = HeapId::Label(Id::from("foo"));
        let access = Access::Shared(1);
        let system_id = None;
        assert!(!reserve_access_map.is_conflicting_reservation(&item, &access, system_id));

        let source1 = SystemId::from("bar");
        let access_map = vec![(
            item.clone(), access.clone()
        )].into_iter();
        
        reserve_access_map.reserve(source1.clone(), access_map);


        assert!(!reserve_access_map.is_conflicting_reservation(&item, &access, system_id));
        assert!(!reserve_access_map.is_conflicting_reservation(&item, &access, Some(&source1)));
        assert!(reserve_access_map.unreserve(&source1, &item, access.clone()).unwrap().is_ok());

        let access_map = vec![(
            item.clone(), Access::Unique
        )].into_iter();
        
        reserve_access_map.reserve(source1.clone(), access_map);

        assert!(reserve_access_map.is_conflicting_reservation(&item, &access, system_id));
        assert!(reserve_access_map.is_conflicting_reservation(&item, &access, Some(&source1)));
        assert!(!reserve_access_map.is_conflicting_reservation(&item, &Access::Unique, Some(&source1)));
        assert!(reserve_access_map.unreserve(&source1, &item, Access::Unique).unwrap().is_ok());
        assert!(!reserve_access_map.is_conflicting_reservation(&item, &access, system_id));
    }

    #[test]
    fn has_conflicting_reservation() {
        let mut reserve_access_map = ReserveAccessMap::default();

        let mut raw_access_map = RawAccessMap::default();
        let system_id = None;
        assert!(!reserve_access_map.has_conflicting_reservation(&raw_access_map, system_id));

        let item = HeapId::Label(Id::from("foo"));
        let access = Access::Unique;

        assert!(raw_access_map.do_access(item.clone(), access.clone()).is_ok());
        assert!(!reserve_access_map.has_conflicting_reservation(&raw_access_map, system_id));
        reserve_access_map.reserve(SystemId::from("bax"), raw_access_map.clone().drain());
        assert!(reserve_access_map.has_conflicting_reservation(&raw_access_map, system_id));
        assert!(reserve_access_map.unreserve(&SystemId::from("bax"), &item, access).unwrap().is_ok());
        reserve_access_map.reserve(SystemId::from("bax"), raw_access_map.clone().drain());
        assert!(!reserve_access_map.has_conflicting_reservation(&raw_access_map, Some(&SystemId::from("bax"))));
    }
}