use std::collections::HashMap;

use crate::{memory::{ResourceId, access_checked_heap::{heap::HeapId, reservation_access_map::ReservationAccessMap}, access_map::Access, errors::{DeResolveError, ResolveError}, memory_domain::MemoryDomain}, system::system_metadata::Source};

#[derive(Debug, Default)]
pub struct RawAccessMap(HashMap<HeapId, Access>);

impl From<ReservationAccessMap> for RawAccessMap {
    fn from(mut value: ReservationAccessMap) -> Self {
        Self(value.drain().collect())
    }
}

impl RawAccessMap {
    pub fn drain(&mut self) -> impl Iterator<Item = (HeapId, Access)> {
        self.0.drain()
    }

    pub fn merge(&mut self, other: impl Iterator<Item = (HeapId, Access)>) {
        self.0.extend(other);
    }

    /// are all resources in self (accesses) also in memory_domain
    pub fn ok_resources(&self, memory_domain: &MemoryDomain) -> bool {
        self.0.keys().all(|heap_id| memory_domain.ok_resource(&ResourceId::Heap(heap_id.clone())))
    }

    /// are all accesses in self ok / do not conflict with the memory domain's accesses
    pub fn ok_accesses(&self, memory_domain: &MemoryDomain, source: Option<&Source>) -> bool {
        self.0.iter().all(|(heap_id, access)| memory_domain.ok_access(&ResourceId::Heap(heap_id.clone()), access, source))
    }

    /// checks if the testing access would conflict with any current access
    pub fn ok_access(&self, testing_heap_id: &HeapId, testing_access: &Access) -> bool {
        if let Some(access) = self.0.get(testing_heap_id) {
            return match (testing_access, access) {
                (Access::Shared(_), Access::Shared(_)) => true,
                    _ => false
            };
        }

        true
    }

    pub fn conflicts(&self, other: &Self) -> bool {
        other.0.iter().any(|(testing_heap_id, testing_access)| {
            !self.ok_access(testing_heap_id, testing_access)
        })
    }

    pub fn deaccess(&mut self, access: &Access, heap_id: &HeapId) -> Result<(), DeResolveError> {
        match self.0.get_mut(heap_id) {
            Some(Access::Shared(n)) => {
                match access {
                    Access::Unique => Err(DeResolveError::AccessMismatch),
                    Access::Shared(m) => { 
                        if m > n {
                            return Err(DeResolveError::AccessDoesNotExist);
                        }

                        *n -= m;

                        if *n == 0 {
                            self.0.remove(heap_id);
                        }

                        Ok(()) 
                    }
                }
            },
            Some(Access::Unique) => {
                match access {
                    Access::Shared(_) => Err(DeResolveError::AccessMismatch),
                    Access::Unique => { self.0.remove(heap_id); Ok(()) }
                }
            },
            None => Err(DeResolveError::AccessDoesNotExist)
        }
    }

    pub fn get_access(&self, resource_id: &HeapId) -> Option<&Access> {
        self.0.get(resource_id)
    }

    /// combine access shared and access unique by matching on access
    // pub fn do_access(&mut self, heap_id: HeapId, access: &Access) -> Result<(), ResolveError> { todo!() }

    pub fn access_shared(&mut self, heap_id: HeapId) -> Result<(), ResolveError> {
        match self.0.entry(heap_id.clone()).or_insert(Access::Shared(0)) {
            Access::Shared(n) => {
                match n.checked_add(1) {
                    Some(new_n) => *n = new_n,
                    None => return Err(ResolveError::TooManyAccesses(ResourceId::from(heap_id))),
                }

                Ok(())
            },
            Access::Unique => Err(ResolveError::ConflictingAccess(ResourceId::from(heap_id)))
        }
    }
    
    pub fn access_unique(&mut self, heap_id: HeapId) -> Result<(), ResolveError> {
        if self.0.contains_key(&heap_id) {
            return Err(ResolveError::ConflictingAccess(ResourceId::from(heap_id)));
        }

        self.0.insert(heap_id, Access::Unique);
        Ok(())
    }
}

#[cfg(test)]
mod raw_access_map_tests {
    use crate::{id::Id, memory::{ResourceId, access_checked_heap::{heap::{HeapId, HeapObject, raw_heap_object::RawHeapObject}, raw_access_map::RawAccessMap}, access_map::Access, memory_domain::MemoryDomain, resource_id::Resource}};

    #[test]
    fn ok_resources() {
        let mut heap_access_map = RawAccessMap::default();
        let memory_domain = MemoryDomain::new();

        assert!(heap_access_map.ok_resources(&memory_domain));
        
        let heap_id = HeapId::Label(Id("foo".to_string()));

        assert!(heap_access_map.access_unique(heap_id.clone()).is_ok());
        assert!(!heap_access_map.ok_resources(&memory_domain));

        let resource_id = ResourceId::Heap(heap_id);
        memory_domain.insert(resource_id, Resource::Heap(HeapObject(RawHeapObject::new(Box::new(1)))));
        assert!(heap_access_map.ok_resources(&memory_domain));
    }

    #[test]
    fn ok_accesses() {
        let mut heap_access_map = RawAccessMap::default();
        let memory_domain = MemoryDomain::new();

        let source = None;

        assert!(heap_access_map.ok_accesses(&memory_domain, source));
        
        let heap_id = HeapId::Label(Id("foo".to_string()));

        assert!(heap_access_map.access_unique(heap_id.clone()).is_ok());
        assert!(!heap_access_map.ok_accesses(&memory_domain, source));

        let resource_id = ResourceId::Heap(heap_id);
        memory_domain.insert(resource_id.clone(), Resource::Heap(HeapObject(RawHeapObject::new(Box::new(1)))));
        assert!(heap_access_map.ok_accesses(&memory_domain, source));
        let _r = memory_domain.get_unique::<i32>(&resource_id, source);
        assert!(!heap_access_map.ok_accesses(&memory_domain, source));
    }

    #[test]
    fn access_shared() {
        let mut raw_access_map = RawAccessMap::default();

        let heap_id = HeapId::Label(Id("foo".to_string()));
        
        assert!(raw_access_map.access_shared(heap_id.clone()).is_ok());
        assert!(raw_access_map.access_shared(heap_id.clone()).is_ok());
        assert!(raw_access_map.access_shared(heap_id.clone()).is_ok());
        assert!(raw_access_map.access_unique(heap_id).is_err());
    }
   
    #[test]
    fn access_unique() {
        let mut raw_access_map = RawAccessMap::default();

        let heap_id = HeapId::Label(Id("foo".to_string()));
        
        assert!(raw_access_map.access_unique(heap_id.clone()).is_ok());
        assert!(raw_access_map.access_unique(heap_id.clone()).is_err());
        assert!(raw_access_map.access_shared(heap_id.clone()).is_err());

        let heap_id2 = HeapId::Label(Id("bar".to_string()));
        assert!(raw_access_map.access_unique(heap_id2.clone()).is_ok());
    }
    
    #[test]
    fn deaccess() {
        let mut raw_access_map = RawAccessMap::default();

        let heap_id = HeapId::Label(Id("foo".to_string()));

        assert!(raw_access_map.access_unique(heap_id.clone()).is_ok());
        assert!(raw_access_map.deaccess(&Access::Unique, &heap_id).is_ok());
        assert!(raw_access_map.access_unique(heap_id.clone()).is_ok());
        assert!(raw_access_map.deaccess(&Access::Unique, &heap_id).is_ok());
        assert!(raw_access_map.access_shared(heap_id.clone()).is_ok());
        assert!(raw_access_map.access_shared(heap_id.clone()).is_ok());
        assert!(raw_access_map.deaccess(&Access::Shared(2), &heap_id).is_ok());
        assert!(raw_access_map.deaccess(&Access::Shared(1), &heap_id).is_err());
        assert!(raw_access_map.access_shared(heap_id.clone()).is_ok());
        assert!(raw_access_map.access_shared(heap_id.clone()).is_ok());
        assert!(raw_access_map.deaccess(&Access::Shared(1), &heap_id).is_ok());
        assert!(raw_access_map.deaccess(&Access::Shared(1), &heap_id).is_ok());
        assert!(raw_access_map.deaccess(&Access::Shared(1), &heap_id).is_err());
        assert!(raw_access_map.access_shared(heap_id.clone()).is_ok());
        assert!(raw_access_map.access_shared(heap_id.clone()).is_ok());
        assert!(raw_access_map.deaccess(&Access::Shared(3), &heap_id).is_err());
        assert!(raw_access_map.deaccess(&Access::Shared(2), &heap_id).is_ok());
        assert!(raw_access_map.access_unique(heap_id.clone()).is_ok());
    }

    #[test]
    fn conflicts() {
        let mut raw_access_map_1 = RawAccessMap::default();
        let mut raw_access_map_2 = RawAccessMap::default();

        assert!(!raw_access_map_1.conflicts(&raw_access_map_2));

        let heap_id = HeapId::Label(Id("foo".to_string()));

        assert!(raw_access_map_1.access_shared(heap_id.clone()).is_ok());

        assert!(!raw_access_map_1.conflicts(&raw_access_map_2));

        assert!(raw_access_map_1.access_shared(heap_id.clone()).is_ok());
        
        assert!(!raw_access_map_1.conflicts(&raw_access_map_2));

        assert!(raw_access_map_2.access_unique(heap_id.clone()).is_ok());
        
        assert!(raw_access_map_1.conflicts(&raw_access_map_2));

        assert!(raw_access_map_2.deaccess(&Access::Unique, &heap_id).is_ok());

        assert!(raw_access_map_2.access_shared(heap_id).is_ok());
        
        assert!(!raw_access_map_1.conflicts(&raw_access_map_2));
    }

    #[test]
    fn conflicts_reverse() {
        let mut raw_access_map_1 = RawAccessMap::default();
        let mut raw_access_map_2 = RawAccessMap::default();

        assert!(!raw_access_map_2.conflicts(&raw_access_map_1));

        let heap_id = HeapId::Label(Id("foo".to_string()));

        assert!(raw_access_map_2.access_shared(heap_id.clone()).is_ok());

        assert!(!raw_access_map_2.conflicts(&raw_access_map_1));

        assert!(raw_access_map_2.access_shared(heap_id.clone()).is_ok());
        
        assert!(!raw_access_map_2.conflicts(&raw_access_map_1));

        assert!(raw_access_map_1.access_unique(heap_id.clone()).is_ok());
        
        assert!(raw_access_map_2.conflicts(&raw_access_map_1));

        assert!(raw_access_map_1.deaccess(&Access::Unique, &heap_id).is_ok());

        assert!(raw_access_map_1.access_shared(heap_id).is_ok());
        
        assert!(!raw_access_map_2.conflicts(&raw_access_map_1));
    }

    #[test]
    fn ok_access() {
        let mut raw_access_map = RawAccessMap::default();

        let heap_id = HeapId::Label(Id("foo".to_string()));
        let heap_id2 = HeapId::Label(Id("bar".to_string()));

        assert!(raw_access_map.ok_access(&heap_id, &Access::Unique));
        assert!(raw_access_map.ok_access(&heap_id, &Access::Shared(1)));

        assert!(raw_access_map.access_unique(heap_id.clone()).is_ok());
        assert!(!raw_access_map.ok_access(&heap_id, &Access::Unique));
        assert!(!raw_access_map.ok_access(&heap_id, &Access::Shared(1)));
        assert!(raw_access_map.deaccess(&Access::Unique, &heap_id).is_ok());
        assert!(raw_access_map.access_shared(heap_id.clone()).is_ok());
        assert!(!raw_access_map.ok_access(&heap_id, &Access::Unique));
        assert!(raw_access_map.ok_access(&heap_id, &Access::Shared(1)));

        assert!(raw_access_map.ok_access(&heap_id2, &Access::Unique));
        assert!(raw_access_map.ok_access(&heap_id2, &Access::Shared(1)));
    }
}