use std::{any::{Any, TypeId}, collections::HashMap, sync::Arc};

use crate::{id::Id, injection::injection_trait::{Injection, MemoryTarget}, memory::{access_checked_heap::heap::{raw_heap_object::RawHeapObject, HeapId, HeapObject }, errors::ResolveError, memory_domain::MemoryDomain, resource_id::Resource}, system::system_metadata::Source};

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
    pub fn new() -> Self {
        Self {
            global_memory: Arc::new(MemoryDomain::new()),
            program_memory: HashMap::new()
        }
    }

    // True if success, False if fail, None if program_id is Invalid
    // pub fn test_accesses<T: Injection>(&self, program_id: Option<&Id>, source: Option<&ResourceId>) -> Option<bool> {
    //     let mut access_map = T::create_access_map();
    //     T::resolve_accesses(&mut access_map, source);
    //     Some(match T::select_memory_target() {
    //         MemoryTarget::Global => access_map.test_accesses(&self.global_memory),
    //         MemoryTarget::Program => access_map.test_accesses(self.program_memory.get(program_id.as_ref()?)?) 
    //     })
    // }

    pub fn end_drop_delay(&self, key: u64, program_id: Option<&Id>) -> Option<()> {
        Some(if let Some(program_id) = program_id {
            self.program_memory.get(program_id)?.end_drop_delay(&key);
        } else {
            self.global_memory.end_drop_delay(&key);
        })
    }

    // True if success, False if fail, None if program_id is Invalid
    pub fn ok_resources<T: Injection>(&self, program_id: Option<&Id>, source: Option<&Source>, resource_id: Option<ResourceId>) -> Option<bool> {
        let mut access_map = T::create_access_map();
        T::resolve_accesses(&mut access_map, source, resource_id);
        Some(match T::select_memory_target() {
            MemoryTarget::Global => access_map.ok_resources(&self.global_memory),
            MemoryTarget::Program => access_map.ok_resources(self.program_memory.get(program_id.as_ref()?)?) 
        })
    }

    // True if success, False if fail, None if program_id is Invalid
    pub fn reserve_accesses<T: Injection>(&self, program_id: Option<&Id>, resource_id: Option<ResourceId>, source: Source) -> Option<bool> {
        let mut access_map = T::create_access_map();
        T::resolve_accesses(&mut access_map, Some(&source), resource_id);

        Some(match T::select_memory_target() {
            MemoryTarget::Global => self.global_memory.reserve_accesses(source, access_map),
            MemoryTarget::Program => self.program_memory.get(program_id.as_ref()?)?.reserve_accesses(source, access_map) 
        })
    }

    // panics 
    pub fn quick_resolve<T: Injection>(&self) -> T::Item<'_> {
        self.resolve::<T>(None, None, None).unwrap().unwrap()
    }

    pub fn resolve<T: Injection>(&self, program_id: Option<&Id>, resource_id: Option<&ResourceId>, source: Option<&Source>) -> Option<Result<T::Item<'_>, ResolveError>> {
        let map = match T::select_memory_target() {
            MemoryTarget::Global => &self.global_memory,
            MemoryTarget::Program => self.program_memory.get(program_id.as_ref()?)?
        };

        Some(map.resolve::<T>(resource_id, source))
    }

    pub fn insert<T: 'static>(&self, program_id: Option<&Id>, resource_id: Option<ResourceId>, resource: T) -> Option<Option<Resource>> {
        let resource: Box<dyn Any> = Box::new(resource);
        Some(if let Some(id) = program_id {
            self.program_memory.get(id)?.insert(resource_id.unwrap_or(ResourceId::from(HeapId::from(TypeId::of::<T>()))), Resource::Heap(HeapObject(RawHeapObject::new(resource))))
        } else {
            self.global_memory.insert(resource_id.unwrap_or(ResourceId::from(HeapId::from(TypeId::of::<T>()))), Resource::Heap(HeapObject(RawHeapObject::new(resource))))
        })
    }
}