use crate::memory::access_checked_resource_map::resource::{raw_resource_map::RawResourceMap, Resource, ResourceId};

#[derive(Debug, Default)]
pub struct ResourceMap {
    lock: parking_lot::RwLock<()>,
    raw_resource_map: RawResourceMap
}

impl ResourceMap {
    /// Safety:
    /// Ensure no concurrent mutable accesses
    pub unsafe fn get<T: 'static>(&self, resource_id: &ResourceId) -> Option<&T> {
        let guard = self.lock.read();
        unsafe { self.raw_resource_map.get(resource_id, guard) }
    }

    /// Safety:
    /// Ensure no concurrent accesses
    pub unsafe fn get_mut<T: 'static>(&self, resource_id: &ResourceId) -> Option<&mut T> {
        let guard = self.lock.read();
        unsafe { self.raw_resource_map.get_mut(resource_id, guard) }
    }

    /// Safety:
    /// Ensure no concurrent accesses
    pub unsafe fn insert(&self, resource_id: ResourceId, resource: Resource) -> Option<Resource> {
        let guard = self.lock.write();
        unsafe { self.raw_resource_map.insert(resource_id, resource, guard) }
    }
}