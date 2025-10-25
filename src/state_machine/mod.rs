use std::sync::Arc;

use threadpool::ThreadPool;

use crate::{id::Id, injection::{injection_primitives::unique::Unique, injection_trait::Injection}, memory::{errors::ResolveError, memory_domain::MemoryDomain, program_memory_map::inner_program_memory_map::Key, resource_id::Resource, Memory, ResourceId}, state_machine::{kernel_registry::KernelSystemRegistry, kernel_systems::{background_processor::{finish_background_processor::FinishBackgroundProcessor, start_background_processor::StartBackgroundProcessor}, blocker_manager::BlockerManager, event_manager::EventManager, processor::Processor, KernelSystem, StoredKernelSystem}, transition_phases::TransitionPhase}, system::system_metadata::Source};

pub mod kernel_systems;
pub mod kernel_registry;
// pub mod blacklist;
pub mod transition_phases;

#[derive(Debug)]
pub struct StateMachine {
    state: Arc<Memory>,
    threadpool: ThreadPool,
    runtime: Arc<tokio::runtime::Runtime>,

    program_id: Id,
    kernel_key: Key,
}

impl StateMachine {
    pub fn new(threads: usize) -> Self {
        let memory = Arc::new(Memory::new());
        
        let key = rand::random();
        let id = Id("_KernelMemory".to_string());

        assert!(memory.insert_program(id.clone(), Arc::new(MemoryDomain::new()), Some(key)), "Kernel must have access to `_KernelMemory`");

        memory.insert(
            Some(&id), 
            None, 
            Some(&key),
            KernelSystemRegistry::default()
        );

        Self {
            state: memory,
            threadpool: ThreadPool::new(threads),
            runtime: Arc::new(tokio::runtime::Runtime::new().unwrap()),

            program_id: id,
            kernel_key: key,
        }
    }

    pub fn load_kernel_system<T: KernelSystem + 'static>(&self, mut kernel_system: T, index: usize) {
        let resource_id = kernel_system.init(&self.state);
        
        let mut kernel_system_registry = self.state.resolve::<Unique<KernelSystemRegistry>>(Some(&self.program_id), None, None, Some(&self.kernel_key)).unwrap().unwrap();
        
        assert!(self.state.insert(Some(&self.program_id), Some(resource_id.clone()), Some(&self.kernel_key), Box::new(kernel_system) as StoredKernelSystem).unwrap().is_none());
        kernel_system_registry.insert(index, resource_id);
    }

    pub fn load_default(&self, processor_threads: usize) {
        let mut finish_background_processor = FinishBackgroundProcessor::default();
        let resource_id = finish_background_processor.init(&self.state);
        let start_background_processor = StartBackgroundProcessor::create_from(&finish_background_processor).unwrap();

        {
            let mut kernel_system_registry = self.state.resolve::<Unique<KernelSystemRegistry>>(Some(&self.program_id), None, None, Some(&self.kernel_key)).unwrap().unwrap();
    
            assert!(self.state.insert(Some(&self.program_id), Some(resource_id.clone()), Some(&self.kernel_key), Box::new(finish_background_processor) as StoredKernelSystem).unwrap().is_none());
            kernel_system_registry.insert(0, resource_id);
        }

        self.load_kernel_system(EventManager, 1);
        self.load_kernel_system(BlockerManager, 1);
        self.load_kernel_system(start_background_processor, 3);

        // Refactor: Make a separate unit struct for the `KernelSystem` trait and the rest are on a separate struct the unit instantiates in init
        // So processor can get the processor_threads from memory/state before hand
        let processor = Processor::new(processor_threads);
        self.load_kernel_system(processor, 2);
    }

    pub fn resolve<T: Injection>(&self, program_id: Option<&Id>, resource_id: Option<&ResourceId>, source: Option<&Source>, key: Option<&Key>) -> Option<Result<T::Item<'_>, ResolveError>> {
        self.state.resolve::<T>(program_id, resource_id, source, key)
    }

    pub fn insert<T: 'static>(&self, program_id: Option<&Id>, resource_id: Option<ResourceId>, key: Option<&Key>, resource: T) -> Option<Option<Resource>> {
        self.state.insert(program_id, resource_id, key, resource)
    }

    pub fn insert_program(&self, program_id: Id, memory_domain: Arc<MemoryDomain>, key: Option<Key>) -> bool {
        self.state.insert_program(program_id, memory_domain, key)
    }

    pub async fn transition(&self) {
        let mut kernel_systems = self.state.resolve::<Unique<KernelSystemRegistry>>(Some(&self.program_id), None, None, Some(&self.kernel_key)).unwrap().unwrap();
        for phase in TransitionPhase::iter_fields() {
            for kernel_systems in kernel_systems.iter() {
                for kernel_system in kernel_systems {
                    println!("Doing: {kernel_system:?}");
                    let mut kernel_system = self.state.resolve::<Unique<StoredKernelSystem>>(Some(&self.program_id), Some(&kernel_system), None, Some(&self.kernel_key)).unwrap().unwrap();
                    // println!("Running");
                    kernel_system.tick(&self.state, phase).await;
                    println!("Finished");
                }
                // for kernel_system in kernel_systems.clone() {
                //     let memory = Arc::clone(&self.state);
                //     let runtime = Arc::clone(&self.runtime);
                //     self.threadpool.execute(move || {
                //         let mut kernel_system = memory.resolve::<Unique<StoredKernelSystem>>(None, Some(&kernel_system), None).unwrap().unwrap();
                //         runtime.block_on(kernel_system.tick(&memory, phase));
                //     });
                // }
    
                // self.threadpool.join();
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