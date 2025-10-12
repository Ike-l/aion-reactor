use std::{any::TypeId, sync::{Arc, Mutex}};

use crate::{injection::{injection_trait::Injection, AccessDropper}, memory::access_checked_resource_map::{access::{access_map::AccessMap, Access}, resource::{resource_map::ResourceMap, Resource, ResourceId}}};

pub mod resource;
pub mod access;

#[derive(Debug)]
pub enum ResolveError {
    ConflictingAccess(ResourceId),
    InvalidProgramId,
    NoResource(ResourceId),
}

// Should be no public way of creating one of these to enforce dropping behaviour by injection types
#[derive(Debug)]
pub struct AccessCheckedResourceMap {
    access_map: Mutex<AccessMap>,
    resource_map: ResourceMap,
}

impl AccessCheckedResourceMap {
    pub fn insert<T: 'static>(&mut self, type_id: TypeId, resource: T) -> Option<Resource> {
        todo!()
    }

    pub fn resolve<T: Injection>(self: &Arc<Self>) -> Result<T::Item<'_>, ResolveError> {
        let r = T::retrieve(&self);
        if let Ok(r) = &r {
            // make sure no panics so there MUST be a dropper
            std::hint::black_box(r.access_dropper());
        }

        r
    }

    // pub crate for now since i only want the dropper to use this
    pub(crate) fn deresolve(&self, access: Access, resource: &ResourceId) -> Option<()> {
        todo!()
    }

    pub fn get_shared<T: 'static>(&self) -> Result<&T, ResolveError> {
        self.access_map.lock().unwrap().access_shared(TypeId::of::<T>())?;
        // Safety:
        // Accesses are tracked
        unsafe { self.resource_map.get().ok_or(ResolveError::NoResource(ResourceId::from(TypeId::of::<T>()))) }
    }

    pub fn get_unique<T: 'static>(&self) -> Result<&mut T, ResolveError> {
        self.access_map.lock().unwrap().access_unique(TypeId::of::<T>())?;
        // Safety:
        // Accesses are tracked
        unsafe { self.resource_map.get_mut().ok_or(ResolveError::NoResource(ResourceId::from(TypeId::of::<T>()))) }
    }
}