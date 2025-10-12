use std::any::{Any, TypeId};

use crate::{id::Id, memory::access_checked_heap::heap::raw_heap_object::RawHeapObject};

pub mod heap;
pub mod raw_heap_object;
pub mod raw_heap;
pub mod inner_heap;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum HeapId {
    RawType(TypeId),
    Label(Id),
}

impl From<TypeId> for HeapId {
    fn from(value: TypeId) -> Self {
        Self::RawType(value)
    }
}

impl From<Id> for HeapId {
    fn from(value: Id) -> Self {
        Self::Label(value)
    }
}

#[derive(Debug)]
pub struct HeapObject(pub RawHeapObject<Box<dyn Any>>);

