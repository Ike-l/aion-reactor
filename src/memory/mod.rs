use std::{any::Any, sync::Arc};

use crate::{ids::{program_id::ProgramId, system_id::SystemId}, injection::injection_trait::{Injection, MemoryTarget}, memory::{access_checked_heap::heap::{HeapObject, raw_heap_object::RawHeapObject }, errors::{InsertError, ReservationError, ResolveError}, memory_domain::MemoryDomain, program_memory_map::{ProgramMemoryMap, inner_program_memory_map::ProgramKey}, resource_id::Resource}, prelude::ResourceId};

pub mod access_checked_heap;
pub mod resource_id;
pub mod memory_domain;
pub mod errors;
pub mod access_map;
pub mod program_memory_map;

#[derive(Debug)]
pub struct Memory {
    program_memory_map: ProgramMemoryMap,
    global_memory: ProgramId,
}

impl Memory {
    pub fn new() -> Self {
        let global_memory = ProgramId::from("_GlobalMemory");
        let program_memory_map = ProgramMemoryMap::default();

        assert!(program_memory_map.insert(global_memory.clone(), Arc::new(MemoryDomain::new()), None));

        Self {
            program_memory_map: program_memory_map,
            global_memory
        }
    }

    pub fn insert_program(&self, program_id: ProgramId, memory_domain: Arc<MemoryDomain>, key: Option<ProgramKey>) -> bool {
        self.program_memory_map.insert(program_id, memory_domain, key)
    }

    /// Safety:
    /// Do not deaccess something unless you actually free the access!
    pub unsafe fn end_drop_delay(&self, key: u64, program_id: Option<&ProgramId>, program_key: Option<&ProgramKey>) -> Option<()> {
        let program_id = if let Some(program_id) = program_id { program_id } else { &self.global_memory };
        
        unsafe { self.program_memory_map.get(program_id, program_key)?.end_drop_delay(&key) };
        Some(())
    }

    // True if success, False if fail, None if program_id is Invalid
    pub fn ok_resources<T: Injection>(&self, program_id: Option<&ProgramId>, system_id: Option<&SystemId>, resource_id: Option<ResourceId>, key: Option<&ProgramKey>) -> Option<bool> {
        let mut access_map = T::create_access_map();
        T::resolve_accesses(&mut access_map, system_id, resource_id);
        
        let program_id = match T::select_memory_target() { 
            MemoryTarget::Global => &self.global_memory,
            MemoryTarget::Program => if let Some(program_id) = program_id.as_ref() { program_id } else { &self.global_memory }
        };

        Some(access_map.ok_resources(self.program_memory_map.get(program_id, key)?))
    }

    pub fn ok_accesses<T: Injection>(&self, program_id: Option<&ProgramId>, system_id: Option<&SystemId>, resource_id: Option<ResourceId>, key: Option<&ProgramKey>) -> Option<bool> {
        let mut access_map = T::create_access_map();
        T::resolve_accesses(&mut access_map, system_id, resource_id);
        
        let program_id = match T::select_memory_target() { 
            MemoryTarget::Global => &self.global_memory,
            MemoryTarget::Program => if let Some(program_id) = program_id.as_ref() { program_id } else { &self.global_memory }
        };

        Some(access_map.ok_accesses(self.program_memory_map.get(program_id, key)?, system_id))
    }

    // True if success, False if fail, None if program_id is Invalid
    pub fn reserve_accesses<T: Injection>(&self, program_id: Option<&ProgramId>, resource_id: Option<ResourceId>, system_id: SystemId, key: Option<&ProgramKey>) -> Option<Result<(), ReservationError>> {
        let mut access_map = T::create_access_map();
        T::resolve_accesses(&mut access_map, Some(&system_id), resource_id);

        let program_id = match T::select_memory_target() { 
            MemoryTarget::Global => &self.global_memory,
            MemoryTarget::Program => if let Some(program_id) = program_id.as_ref() { program_id } else { &self.global_memory }   
        };

        Some(self.program_memory_map.get(program_id, key)?.reserve_accesses(system_id, access_map))
    }

    /// doesnt check for resource (so works for empty)
    pub fn reserve_current_accesses<T: Injection>(&self, program_id: Option<&ProgramId>, resource_id: Option<ResourceId>, system_id: SystemId, key: Option<&ProgramKey>) -> Option<Result<(), ReservationError>> {
        let mut access_map = T::create_access_map();
        T::resolve_accesses(&mut access_map, Some(&system_id), resource_id);

        let program_id = match T::select_memory_target() { 
            MemoryTarget::Global => &self.global_memory,
            MemoryTarget::Program => if let Some(program_id) = program_id.as_ref() { program_id } else { &self.global_memory }   
        };
        
        Some(self.program_memory_map.get_or_default(program_id.clone(), key).reserve_current_accesses(system_id, access_map))
    }

    pub fn try_integrate_reservations(&self, other: Self, system_id: SystemId) -> Option<ReservationError> {
        match self.program_memory_map.atomic_reservations(other.program_memory_map, &system_id) {
            Ok(_) => None,
            Err(err) => Some(err),
        }
    }

    pub fn resolve<T: Injection>(&self, program_id: Option<&ProgramId>, resource_id: Option<&ResourceId>, system_id: Option<&SystemId>, key: Option<&ProgramKey>) -> Option<Result<T::Item<'_>, ResolveError>> {
        let map = match T::select_memory_target() {
            MemoryTarget::Global => self.program_memory_map.get(&self.global_memory, key)?,
            MemoryTarget::Program => {
                if let Some(program_id) = program_id {
                    self.program_memory_map.get(program_id, key)?
                } else {
                    self.program_memory_map.get(&self.global_memory, key)?
                }
            } 
        };

        Some(map.resolve::<T>(resource_id, system_id))
    }

    /// None: No Program Found
    /// 
    /// Some/Err: InsertError
    /// 
    /// Some/Ok/None: No ResourceId/Resource Existed
    /// 
    /// Some/Ok/Some: Some ResourceId/Resource Existed
    pub fn insert<T: 'static>(&self, program_id: Option<&ProgramId>, resource_id: Option<ResourceId>, key: Option<&ProgramKey>, resource: T) -> Option<Result<Option<Resource>, InsertError>> {
        let resource: Box<dyn Any> = Box::new(resource);

        let program_id = match program_id {
            Some(program_id) => program_id,
            None => &self.global_memory,
        };

        Some(
            self.program_memory_map.get(program_id, key)?
                .insert(
                    resource_id
                        .unwrap_or(
                            ResourceId::from_raw_heap::<T>()), 
                            Resource::Heap(HeapObject(RawHeapObject::new(resource))
                    )
            )            
        )
    }

    pub fn contains_resource(&self, program_id: Option<&ProgramId>, resource_id: &ResourceId, key: Option<&ProgramKey>) -> Option<bool> {
        let program_id = match program_id {
            Some(program_id) => program_id,
            None => &self.global_memory,
        };

        Some(self.program_memory_map.get(program_id, key)?.ok_resource(resource_id))
    }
}