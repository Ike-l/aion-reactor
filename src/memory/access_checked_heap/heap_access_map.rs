use std::collections::HashMap;

use crate::memory::{access_checked_heap::heap::HeapId, access_map::Access, errors::{DeResolveError, ResolveError}, ResourceId};


#[derive(Debug, Default)]
pub struct HeapAccessMap(HashMap<HeapId, Access>);

impl HeapAccessMap {
    pub fn drain(&mut self) -> impl Iterator<Item = (HeapId, Access)> {
        self.0.drain()
    }

    pub fn conflicts(&self, other: &Self) -> bool {
        other.0.iter().any(|(ty, acc)| {
            if let Some(access) = self.0.get(ty) {
                match (acc, access) {
                    (Access::Shared(_), Access::Shared(_)) => false,
                    _ => true
                }
            } else {
                false
            }
        })
    }

    pub fn deaccess(&mut self, access: Access, heap_id: &HeapId) -> Result<(), DeResolveError> {
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

    pub fn access(&self, resource_id: &HeapId) -> Option<&Access> {
        self.0.get(resource_id)
    }

    pub fn access_shared<T: Into<HeapId>>(&mut self, heap_id: T) -> Result<(), ResolveError> {
        let heap_id = heap_id.into();
        match self.0.entry(heap_id.clone()).or_insert(Access::Shared(0)) {
            Access::Shared(n) => Ok(*n += 1),
            Access::Unique => Err(ResolveError::ConflictingAccess(ResourceId::from(heap_id)))
        }
    }

    pub fn access_unique<T: Into<HeapId>>(&mut self, heap_id: T) -> Result<(), ResolveError> {
        let heap_id = heap_id.into();
        if self.0.contains_key(&heap_id) {
            return Err(ResolveError::ConflictingAccess(ResourceId::from(heap_id)));
        }

        self.0.insert(heap_id, Access::Unique);
        Ok(())
    }
}