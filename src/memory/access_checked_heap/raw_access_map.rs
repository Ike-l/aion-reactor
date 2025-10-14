use std::collections::HashMap;

use crate::memory::{access_checked_heap::heap::HeapId, access_map::Access, errors::{DeResolveError, ResolveError}, memory_domain::MemoryDomain, ResourceId};

#[derive(Debug, Default)]
pub struct RawAccessMap(HashMap<HeapId, Access>);

impl RawAccessMap {
    pub fn drain(&mut self) -> impl Iterator<Item = (HeapId, Access)> {
        self.0.drain()
    }

    pub fn merge(&mut self, other: Self) {
        self.0.extend(other.0);
    }

    pub fn ok_resources(&self, memory_domain: &MemoryDomain) -> bool {
        self.0.keys().all(|heap_id| memory_domain.ok_resource(&ResourceId::Heap(heap_id.clone())))
    }

    pub fn ok_accesses(&self, memory_domain: &MemoryDomain) -> bool {
        self.0.iter().all(|(heap_id, access)| memory_domain.ok_access(&ResourceId::Heap(heap_id.clone()), access))
    }

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
                    Access::Shared(m) => { *n -= m; Ok(()) }
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

    pub fn access_shared(&mut self, heap_id: HeapId) -> Result<(), ResolveError> {
        match self.0.entry(heap_id.clone()).or_insert(Access::Shared(0)) {
            Access::Shared(n) => Ok(*n += 1),
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