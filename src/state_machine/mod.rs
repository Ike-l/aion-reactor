use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

use tracing::{Level, event, span};

use crate::prelude::{Injection, InsertError, KernelSystemRegistry, Memory, MemoryDomain, ProgramId, ProgramKey, ResolveError, Resource, ResourceId, StoredKernelSystem, SystemId, Unique};

pub mod kernel_systems;
pub mod kernel_registry;
pub mod kernel_builder;

#[derive(Debug)]
pub struct StateMachine {
    memory: Arc<Memory>,
    program_id: ProgramId,
    kernel_key: ProgramKey,
    
    current_tick: AtomicUsize,
}


impl StateMachine {
    pub fn new() -> Self {
        let memory = Arc::new(Memory::new());
        
        let key = rand::random();
        let id = ProgramId::from("_KernelMemory");

        assert!(memory.insert_program(id.clone(), Arc::new(MemoryDomain::new()), Some(key)), "Kernel must have access to `_KernelMemory`");

        memory.insert(
            Some(&id), 
            None, 
            Some(&key),
            KernelSystemRegistry::default()
        );

        Self {
            memory,
            program_id: id,
            kernel_key: key,
            current_tick: AtomicUsize::new(0),
        }
    }

    fn insert_stored_system(
        &self,
        system_id: SystemId,
        kernel_system: StoredKernelSystem,
        ordering_index: usize
    ) {
        let mut kernel_system_registry = self.memory.resolve::<Unique<KernelSystemRegistry>>(Some(&self.program_id), None, None, Some(&self.kernel_key)).unwrap().unwrap();
        let resource_id = ResourceId::from_labelled_heap(system_id.into_id());

        assert!(
            self.memory.insert(
                Some(&self.program_id), 
                Some(resource_id.clone()), 
                Some(&self.kernel_key), 
                kernel_system
            ).unwrap().unwrap().is_none()
        );

        kernel_system_registry.insert(ordering_index, resource_id);
    }

    pub fn resolve<T: Injection>(&self, program_id: Option<&ProgramId>, resource_id: Option<&ResourceId>, source: Option<&SystemId>, key: Option<&ProgramKey>) -> Option<Result<T::Item<'_>, ResolveError>> {
        self.memory.resolve::<T>(program_id, resource_id, source, key)
    }

    pub fn insert<T: 'static>(&self, program_id: Option<&ProgramId>, resource_id: Option<ResourceId>, key: Option<&ProgramKey>, resource: T) -> Option<Result<Option<Resource>, InsertError>> {
        self.memory.insert(program_id, resource_id, key, resource)
    }

    pub fn insert_program(&self, program_id: ProgramId, memory_domain: Arc<MemoryDomain>, key: Option<ProgramKey>) -> bool {
        self.memory.insert_program(program_id, memory_domain, key)
    }

    // could make it async but then Processor run time weird stuff so idk
    pub fn tick(&self) {
        let current_tick = self.current_tick.fetch_add(1, Ordering::Acquire);

        let span = span!(Level::INFO, "Tick", current_tick=current_tick);
        let _enter = span.enter();

        event!(Level::INFO, "Start Tick");

        let mut kernel_systems = self.memory.resolve::<Unique<KernelSystemRegistry>>(Some(&self.program_id), None, None, Some(&self.kernel_key)).unwrap().unwrap();
        for kernel_systems in kernel_systems.iter() {
            // could parallelise later?
            for kernel_system in kernel_systems {
                let mut kernel_system = self.memory.resolve::<Unique<StoredKernelSystem>>(Some(&self.program_id), Some(&kernel_system), None, Some(&self.kernel_key)).unwrap().unwrap();
                let span = span!(Level::DEBUG, "Kernel System Tick", kernel_system = ?kernel_system.system_id().into_id());
                let _enter = span.enter();

                event!(Level::DEBUG, "Started");
                
                // make async? but if i do its quite misleading cause it does blocking work
                pollster::block_on(kernel_system.tick(
                    &self.memory, 
                    self.program_id.clone(), 
                    self.kernel_key.clone()
                ));

                event!(Level::DEBUG, "Finished");
            }
        }
    }
}

#[cfg(test)]
mod state_machine_tests {
    // test tick
    // test insert resource + [conflict/no conflict]
    // test get resource + [exist/no exist]
}