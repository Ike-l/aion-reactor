use std::collections::HashMap;

use crate::memory::access_checked_resource_map::resource::{Resource, ResourceId};

#[derive(Debug, Default)]
pub struct InnerResourceMap {
    resources: HashMap<ResourceId, Resource>
}

impl InnerResourceMap {
    /// Safety:
    /// Ensure no concurrent accesses
    pub unsafe fn get<T: 'static>(&self, resource_id: &ResourceId) -> Option<&T> {
        unsafe {
            self.resources
                .get(resource_id)
                .map(|cell| & *cell.0.get())
                .and_then(|boxed| boxed.downcast_ref::<T>())
        }
    }

    /// Safety:
    /// Ensure no concurrent accesses
    pub unsafe fn get_mut<T: 'static>(&self, resource_id: &ResourceId) -> Option<&mut T> {
        unsafe {
            self.resources
                .get(resource_id)
                .map(|cell| &mut *cell.0.get())
                .and_then(|boxed| boxed.downcast_mut::<T>())
        }
    }

    pub unsafe fn insert(&mut self, resource_id: ResourceId, resource: Resource) -> Option<Resource> {
        self.resources.insert(resource_id, resource)
    }
}