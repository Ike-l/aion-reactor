use std::{any::{Any, TypeId}, collections::HashMap, sync::Arc};

use crate::{id::Id, injection::injection_trait::{Injection, MemoryTarget}, memory::access_checked_resource_map::{heap::{raw_heap_object::RawHeapObject, HeapObject, }, AccessCheckedHeap, ResolveError}};

pub mod access_checked_resource_map;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ResourceId {
    Label(Id),
    Heap(TypeId),
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

#[derive(Debug)]
pub struct Memory {
    global_memory: Arc<AccessCheckedHeap>,
    program_memory: HashMap<Id, Arc<AccessCheckedHeap>>
}

impl Memory {
    #[allow(dead_code)]
    #[cfg(test)]
    pub(crate) fn new() -> Self {
        Self {
            global_memory: Arc::new(AccessCheckedHeap::new()),
            program_memory: HashMap::new()
        }
    }

    pub fn resolve<T: Injection>(&self, program_id: Option<Id>, resource_id: Option<ResourceId>) -> Option<Result<T::Item<'_>, ResolveError>> {
        let map = match T::select_memory_target() {
            MemoryTarget::Global => &self.global_memory,
            MemoryTarget::Program => self.program_memory.get(&program_id?)?
        };

        Some(map.resolve::<T>(resource_id))
    }

    pub fn insert<T: 'static>(&mut self, program_id: Option<Id>, resource_id: Option<ResourceId>, resource: T) -> Option<Option<HeapObject>> {
        let resource: Box<dyn Any> = Box::new(resource);
        Some(if let Some(id) = program_id {
            self.program_memory.get(&id)?.insert(resource_id.unwrap_or(ResourceId::Heap(TypeId::of::<T>())), HeapObject(RawHeapObject::new(resource)))
        } else {
            self.global_memory.insert(resource_id.unwrap_or(ResourceId::Heap(TypeId::of::<T>())), HeapObject(RawHeapObject::new(resource)))
        })
    }
}