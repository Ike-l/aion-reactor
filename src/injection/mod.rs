pub mod injection_trait;
pub mod injection_primitives;

use std::sync::Arc;

use crate::{injection::injection_trait::Injection, memory::{access_checked_heap::access::access_map::HeapAccessMap, memory_domain::MemoryDomain, ResourceId}};

pub trait AccessDropper {
    fn access_dropper(&self) -> &AccessDeResolver; 
}

pub struct AccessDeResolver {
    memory_domain: Arc<MemoryDomain>,
    access_map: HeapAccessMap
}

macro_rules! retrieve {
    ($memory_domain:ident) => { { AccessDeResolver::new::<Self>(Arc::clone($memory_domain)) } };
}

pub(crate) use retrieve;

macro_rules! resolve {
    ($memory:ident, $program_id:ident, $resource_id:ident) => { { Ok($memory.resolve::<Self>($program_id, $resource_id).unwrap_or_else(|| Err(ResolveError::InvalidProgramId))) } };
}

pub(crate) use resolve;

impl AccessDeResolver {
    fn new<T: Injection>(memory_domain: Arc<MemoryDomain>) -> Self {
        let mut access_map = HeapAccessMap::default();
        T::resolve_accesses(&mut access_map);
        Self { memory_domain: memory_domain, access_map }
    }
}

impl Drop for AccessDeResolver {
    fn drop(&mut self) {
        for (resource, access) in self.access_map.drain() {
            self.memory_domain.deresolve(access, &ResourceId::Heap(resource)).unwrap();
        }
    }
}