use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

use tracing::{Level, event, span};

use crate::prelude::{BlockerManager, BlockingProcessor, DelayManager, EventManager, ExecutableManager, FinishNonBlockingProcessor, Injection, InsertError, KernelSystem, KernelSystemRegistry, Memory, MemoryDomain, ProgramId, ProgramKey, ReadOnlyProcessor, ResolveError, Resource, ResourceId, StartNonBlockingProcessor, StoredKernelSystem, SystemId, Unique};

pub mod kernel_systems;
pub mod kernel_registry;

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

    fn insert_system<T: KernelSystem + 'static>(
        &self, 
        system_id: SystemId,
        mut kernel_system: T, 
        ordering_index: usize
    ) {
        kernel_system.init(&self.memory, &self.program_id, &self.kernel_key);

        let mut kernel_system_registry = self.memory.resolve::<Unique<KernelSystemRegistry>>(Some(&self.program_id), None, None, Some(&self.kernel_key)).unwrap().unwrap();
        
        let resource_id = ResourceId::from_labelled_heap(system_id.into_id());

        assert!(
            self.memory.insert(
                Some(&self.program_id), 
                Some(resource_id.clone()), 
                Some(&self.kernel_key), 
                Box::new(kernel_system) as StoredKernelSystem
            ).unwrap().unwrap().is_none()
        );

        kernel_system_registry.insert(ordering_index, resource_id);
    }

    pub fn load_default(&self, processor_threads: usize) {
        let span = span!(Level::DEBUG, "Loading Default Systems");
        let _enter = span.enter();

        event!(Level::DEBUG, "Started");

        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let threadpool = threadpool::ThreadPool::new(processor_threads);

        assert!(self.memory.insert(
            Some(&self.program_id), 
            None, 
            Some(&self.kernel_key), 
            Arc::new(rt)
        ).unwrap().is_ok());

        assert!(self.memory.insert(
            Some(&self.program_id), 
            None, 
            Some(&self.kernel_key), 
            threadpool
        ).unwrap().is_ok());

        let system_id = FinishNonBlockingProcessor.system_id();
        self.insert_system(system_id, FinishNonBlockingProcessor, 0);

        let system_id = EventManager.system_id();
        self.insert_system(system_id, EventManager, 1);

        let system_id = ExecutableManager.system_id();
        self.insert_system(system_id, ExecutableManager, 2);

        let system_id = DelayManager.system_id();
        self.insert_system(system_id, DelayManager, 3);

        let system_id = BlockerManager.system_id();
        self.insert_system(system_id, BlockerManager, 4);

        let system_id = BlockingProcessor.system_id();
        self.insert_system(system_id, BlockingProcessor, 5);

        let system_id = ReadOnlyProcessor.system_id();
        self.insert_system(system_id, ReadOnlyProcessor, 6);

        let system_id = StartNonBlockingProcessor.system_id();
        self.insert_system(system_id, StartNonBlockingProcessor, 7);
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
    pub fn transition(&self) {
        let current_tick = self.current_tick.fetch_add(1, Ordering::Acquire);
        let span = span!(Level::INFO, "Transition", current_tick=current_tick);
        let _enter = span.enter();
        event!(Level::INFO, "Start Transition");
        let mut kernel_systems = self.memory.resolve::<Unique<KernelSystemRegistry>>(Some(&self.program_id), None, None, Some(&self.kernel_key)).unwrap().unwrap();
        for kernel_systems in kernel_systems.iter() {
            for kernel_system in kernel_systems {
                let mut kernel_system = self.memory.resolve::<Unique<StoredKernelSystem>>(Some(&self.program_id), Some(&kernel_system), None, Some(&self.kernel_key)).unwrap().unwrap();
                let span = span!(Level::DEBUG, "Kernel System", kernel_system = ?kernel_system.system_id().into_id());
                let _enter = span.enter();

                event!(Level::DEBUG, status="Ticking");
                pollster::block_on(kernel_system.tick(&self.memory, self.program_id.clone(), self.kernel_key.clone()));
                event!(Level::DEBUG, status="Ticked");
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