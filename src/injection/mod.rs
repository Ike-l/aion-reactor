pub mod injection_trait;
pub mod injection_primitives;

use std::sync::Arc;

use crate::{injection::injection_trait::Injection, memory::access_checked_resource_map::{access::access_map::AccessMap, AccessCheckedHeap}};

pub trait AccessDropper {
    fn access_dropper(&self) -> &AccessDeResolver; 
}

pub struct AccessDeResolver {
    resource_map: Arc<AccessCheckedHeap>,
    access_map: AccessMap
}

macro_rules! retrieve {
    ($resource_map:ident) => { { AccessDeResolver::new::<Self>(Arc::clone($resource_map)) } };
}

pub(crate) use retrieve;

macro_rules! resolve {
    ($memory:ident, $program_id:ident, $resource_id:ident) => { { Ok($memory.resolve::<Self>($program_id, $resource_id).unwrap_or_else(|| Err(ResolveError::InvalidProgramId))) } };
}

pub(crate) use resolve;

impl AccessDeResolver {
    fn new<T: Injection>(resource_map: Arc<AccessCheckedHeap>) -> Self {
        let mut access_map = AccessMap::default();
        T::resolve_accesses(&mut access_map);
        Self { resource_map, access_map }
    }
}

impl Drop for AccessDeResolver {
    fn drop(&mut self) {
        for (resource, access) in self.access_map.drain() {
            self.resource_map.deresolve(access, &resource).unwrap();
        }
    }
}