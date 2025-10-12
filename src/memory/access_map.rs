use crate::memory::{access_checked_heap::heap_access_map::HeapAccessMap, ResourceId};

#[derive(Debug)]
pub enum Access {
    Unique,
    Shared(usize)
}

pub enum AccessMap {
    Heap(HeapAccessMap)
}

impl AccessMap {
    pub fn drain(&mut self) -> impl Iterator<Item = (ResourceId, Access)> {
        match self {
            Self::Heap(access_map) => access_map.drain().map(|(heap_id, access)| (ResourceId::from(heap_id), access))
        }
    }
}