use std::cell::UnsafeCell;

use crate::memory::access_checked_resource_map::resource::{inner_resource_map::InnerResourceMap, Resource, ResourceId};

#[derive(Debug, Default)]
pub struct RawResourceMap {
    resources: UnsafeCell<InnerResourceMap>
}

impl RawResourceMap {
    /// Safety:
    /// Ensure no concurrent accesses
    unsafe fn get_inner_resource_map(&self) -> &InnerResourceMap {
        unsafe { & *self.resources.get() }
    }

    /// Safety:
    /// Ensure no concurrent accesses
    unsafe fn get_mut_inner_resource_map(&self) -> &mut InnerResourceMap {
        unsafe { &mut *self.resources.get() }
    }

    /// Safety:
    /// Ensure no concurrent accesses
    pub unsafe fn get<T: 'static>(&self, resource_id: &ResourceId, _guard: parking_lot::RwLockReadGuard<()>) -> Option<&T> {
        unsafe { self.get_inner_resource_map().get(resource_id) }        
    }

    /// Safety:
    /// Ensure no concurrent accesses
    pub unsafe fn get_mut<T: 'static>(&self, resource_id: &ResourceId, _guard: parking_lot::RwLockReadGuard<()>) -> Option<&mut T> {
        unsafe { self.get_inner_resource_map().get_mut(resource_id) }
    }

    pub unsafe fn insert(&self, resource_id: ResourceId, resource: Resource, _guard: parking_lot::RwLockWriteGuard<()>) -> Option<Resource> {
        unsafe { self.get_mut_inner_resource_map().insert(resource_id, resource) }
    }
}