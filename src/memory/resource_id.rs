use std::any::TypeId;

use crate::{id::Id, memory::access_checked_heap::heap::HeapObject};

pub enum Resource {
    Heap(HeapObject),
    // Stack
    // ECS
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ResourceId {
    Label(Id),
    Heap(TypeId),
    // Stack
    // ECS
}

impl From<TypeId> for ResourceId {
    fn from(value: TypeId) -> Self {
        Self::Heap(value)
    }
}

impl From<Id> for ResourceId {
    fn from(value: Id) -> Self {
        Self::Label(value)
    }
}