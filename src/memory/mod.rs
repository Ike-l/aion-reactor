use std::{any::{Any, TypeId}, collections::HashMap, sync::Arc};

use crate::{id::Id, injection::injection_trait::{Injection, MemoryTarget}, memory::{access_checked_heap::heap::{raw_heap_object::RawHeapObject, HeapId, HeapObject }, errors::ResolveError, memory_domain::MemoryDomain, resource_id::Resource}};

pub mod access_checked_heap;
pub mod resource_id;
pub mod memory_domain;
pub mod errors;
pub mod access_map;

pub use resource_id::ResourceId;

#[derive(Debug)]
pub struct Memory {
    global_memory: Arc<MemoryDomain>,
    program_memory: HashMap<Id, Arc<MemoryDomain>>
}

impl Memory {
    #[allow(dead_code)]
    #[cfg(test)]
    pub(crate) fn new() -> Self {
        Self {
            global_memory: Arc::new(MemoryDomain::new()),
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

    pub fn insert<T: 'static>(&mut self, program_id: Option<Id>, resource_id: Option<ResourceId>, resource: T) -> Option<Option<Resource>> {
        let resource: Box<dyn Any> = Box::new(resource);
        Some(if let Some(id) = program_id {
            self.program_memory.get(&id)?.insert(resource_id.unwrap_or(ResourceId::from(HeapId::from(TypeId::of::<T>()))), Resource::Heap(HeapObject(RawHeapObject::new(resource))))
        } else {
            self.global_memory.insert(resource_id.unwrap_or(ResourceId::from(HeapId::from(TypeId::of::<T>()))), Resource::Heap(HeapObject(RawHeapObject::new(resource))))
        })
    }
}