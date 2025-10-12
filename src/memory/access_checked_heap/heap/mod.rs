use std::any::Any;

use crate::memory::access_checked_heap::heap::raw_heap_object::RawHeapObject;

pub mod heap;
pub mod raw_heap_object;
pub mod raw_heap;
pub mod inner_heap;

#[derive(Debug)]
pub struct HeapObject(pub RawHeapObject<Box<dyn Any>>);

