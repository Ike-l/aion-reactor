use crate::memory::{access_checked_heap::heap_access_map::HeapAccessMap, memory_domain::MemoryDomain, ResourceId};

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

    pub fn ok_resources(&self, memory_domain: &MemoryDomain) -> bool {
        match self {
            Self::Heap(access_map) => access_map.ok_resources(memory_domain)
        }
    }

    pub fn ok_accesses(&self, memory_domain: &MemoryDomain) -> bool {
        match self {
            Self::Heap(access_map) => access_map.ok_accesses(memory_domain)
        }
    }
}