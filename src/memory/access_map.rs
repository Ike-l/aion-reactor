use crate::{memory::{ResourceId, access_checked_heap::reservation_access_map::ReservationAccessMap, memory_domain::MemoryDomain}, system::system_metadata::Source};

#[derive(Debug, Clone)]
pub enum Access {
    Unique,
    Shared(usize)
}

impl Access {
    pub fn is_semantically_different(&self, other: &Self) -> bool {
        match (self, other) {
            (Access::Unique, Access::Unique) |
            (Access::Shared(_), Access::Shared(_)) => false,
            (Access::Unique, Access::Shared(_)) |
            (Access::Shared(_), Access::Unique) => true
        }
    }
}

#[derive(Debug)]
pub enum AccessMap {
    Heap(ReservationAccessMap)
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

    pub fn ok_accesses(&self, memory_domain: &MemoryDomain, source: Option<&Source>) -> bool {
        match self {
            Self::Heap(access_map) => access_map.ok_accesses(memory_domain, source)
        }
    }
}