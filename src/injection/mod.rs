pub mod injection_trait;
pub mod injection_primitives;

use std::sync::Arc;

use crate::{injection::injection_trait::Injection, memory::access_checked_resource_map::{access::access_map::AccessMap, AccessCheckedResourceMap}};

pub trait AccessDropper {
    fn access_dropper(&self) -> &AccessDeResolver; 
}

pub struct AccessDeResolver {
    resource_map: Arc<AccessCheckedResourceMap>,
    access_map: AccessMap
}

impl AccessDeResolver {
    pub fn new<T: Injection>(resource_map: Arc<AccessCheckedResourceMap>) -> Self {
        let mut access_map = AccessMap::default();
        T::resolve_accesses(&mut access_map);
        Self { resource_map, access_map }
    }
}

impl Drop for AccessDeResolver {
    fn drop(&mut self) {
        for (resource, access) in self.access_map.drain() {
            self.resource_map.deresolve(access, &resource);
        }
    }
}