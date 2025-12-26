use std::any::TypeId;

use crate::{memory::access_checked_heap::heap::{HeapId, HeapObject}};

use crate::prelude::Id;

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
    // Stack (arena allocator?),
    // #[cfg(feature = "ecs")]
    // ECS(EntityId)
}

impl ResourceId {
    pub fn from_raw_heap<T: 'static>() -> Self {
        Self::Heap(
            HeapId::RawType(TypeId::of::<T>())
        )
    }

    pub fn from_labelled_heap<T>(id: T) -> Self 
    where Id: From<T>
    {
        Self::Heap(HeapId::Label(id.into()))
    }
}
