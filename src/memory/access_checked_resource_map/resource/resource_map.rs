use std::{any::TypeId, collections::HashMap};

use crate::memory::access_checked_resource_map::resource::{Resource, ResourceId};

#[derive(Debug, Default)]
pub struct ResourceMap {
    resources: HashMap<ResourceId, Resource>
}

impl ResourceMap {
    /// Safety:
    /// Ensure no concurrent mutable accesses
    pub unsafe fn get<T: 'static>(&self) -> Option<&T> {
        unsafe {
            self.resources
                .get(&TypeId::of::<T>().into())
                .map(|cell| & *cell.0.get())
                .and_then(|boxed| boxed.downcast_ref::<T>())
        }
    }

    /// Safety:
    /// Ensure no concurrent accesses
    pub unsafe fn get_mut<T: 'static>(&self) -> Option<&mut T> {
        unsafe {
            self.resources
                .get(&TypeId::of::<T>().into())
                .map(|cell| &mut *cell.0.get())
                .and_then(|boxed| boxed.downcast_mut::<T>())
        }
    }
}