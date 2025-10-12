use std::{any::{Any, TypeId}, collections::HashMap, sync::Arc};

use crate::{id::Id, injection::injection_trait::{Injection, MemoryTarget}, memory::access_checked_resource_map::{resource::{raw_resource::RawResource, Resource, ResourceId}, AccessCheckedResourceMap, ResolveError}};

pub mod access_checked_resource_map;

#[derive(Debug)]
pub struct Memory {
    global_memory: Arc<AccessCheckedResourceMap>,
    program_memory: HashMap<Id, Arc<AccessCheckedResourceMap>>
}

impl Memory {
    #[cfg(test)]
    pub(crate) fn new() -> Self {
        Self {
            global_memory: Arc::new(AccessCheckedResourceMap::new()),
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
            self.program_memory.get(&id)?.insert(resource_id.unwrap_or(ResourceId::RawTypeId(TypeId::of::<T>())), Resource(RawResource::new(resource)))
        } else {
            self.global_memory.insert(resource_id.unwrap_or(ResourceId::RawTypeId(TypeId::of::<T>())), Resource(RawResource::new(resource)))
        })
    }
}