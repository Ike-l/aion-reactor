use std::collections::HashSet;

use crate::memory::{access_checked_heap::heap_access_map::HeapAccessMap, memory_domain::MemoryDomain, Memory, ResourceId};

#[derive(Debug)]
pub enum Access {
    Unique,
    Shared(usize)
}

#[derive(Debug)]
pub enum AccessMap {
    Heap(HeapAccessMap)
}

impl AccessMap {
    pub fn drain(&mut self) -> impl Iterator<Item = (ResourceId, Access)> {
        match self {
            Self::Heap(access_map) => access_map.drain().map(|(heap_id, access)| (ResourceId::from(heap_id), access))
        }
    }

    pub fn test_resources(&self, memory_domain: &MemoryDomain) -> bool {
        match self {
            Self::Heap(access_map) => access_map.test_resources(memory_domain)
        }
    }

    pub fn test_accesses(&self, memory_domain: &MemoryDomain) -> bool {
        match self {
            Self::Heap(access_map) => access_map.test_accesses(memory_domain)
        }
    }
}