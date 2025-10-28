use crate::memory::access_checked_heap::heap::{HeapId, HeapObject};

pub enum Resource {
    Heap(HeapObject),
    // Stack,
    // ECS
}

impl Resource {
    #[cfg(test)]
    pub fn dummy(value: i32) -> Self {
        Self::Heap(HeapObject::dummy(value))
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ResourceId {
    Heap(HeapId),
    // Stack,
    // ECS
}

impl From<HeapId> for ResourceId {
    fn from(value: HeapId) -> Self {
        Self::Heap(value)
    }
}
