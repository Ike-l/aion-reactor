use std::{collections::HashMap, sync::Arc};

use crate::{id::Id, injection::injection_trait::{Injection, MemoryTarget}, memory::access_checked_resource_map::{resource::Resource, AccessCheckedResourceMap, ResolveError}};

pub mod access_checked_resource_map;

#[derive(Debug)]
pub struct Memory {
    global_memory: Arc<AccessCheckedResourceMap>,
    program_memory: HashMap<Id, Arc<AccessCheckedResourceMap>>
}

impl Memory {
    pub fn resolve<T: Injection>(&self, program_id: Option<Id>) -> Option<Result<T::Item<'_>, ResolveError>> {
        let map = match T::select_memory_target() {
            MemoryTarget::Global => &self.global_memory,
            MemoryTarget::Program => self.program_memory.get(&program_id?)?
        };

        Some(map.resolve::<T>())
    }

    pub fn insert<T: 'static>(&mut self, program_id: Option<Id>, resource: T) -> Option<Option<Resource>> {
        todo!()
    }
}