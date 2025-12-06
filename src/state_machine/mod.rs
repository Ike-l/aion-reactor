use std::sync::Arc;

use crate::{id::Id, injection::{injection_primitives::unique::Unique, injection_trait::Injection}, memory::{Memory, ResourceId, errors::{InsertError, ResolveError}, memory_domain::MemoryDomain, program_memory_map::inner_program_memory_map::Key, resource_id::Resource}, state_machine::{kernel_registry::KernelSystemRegistry, kernel_systems::{KernelSystem, StoredKernelSystem, background_processor::{finish_background_processor::FinishBackgroundProcessor, start_background_processor::StartBackgroundProcessor}, blocker_manager::BlockerManager, read_only_processor::ReadOnlyProcessorManager, delay_manager::DelayManager, event_manager::EventManager, executable_manager::ExecutableManager, processor::Processor}, }, system::system_metadata::Source};

pub mod kernel_systems;
pub mod kernel_registry;

#[derive(Debug)]
pub struct StateMachine {
    state: Arc<Memory>,
    program_id: Id,
    kernel_key: Key,
}


impl StateMachine {
    pub fn new() -> Self {
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
            program_id: id,
            kernel_key: key,
        }
    }

    pub fn load_kernel_system<T: KernelSystem + 'static>(&self, mut kernel_system: T, index: usize) {
        let resource_id = kernel_system.init(&self.state);
        
        let mut kernel_system_registry = self.state.resolve::<Unique<KernelSystemRegistry>>(Some(&self.program_id), None, None, Some(&self.kernel_key)).unwrap().unwrap();
        
        assert!(self.state.insert(Some(&self.program_id), Some(resource_id.clone()), Some(&self.kernel_key), Box::new(kernel_system) as StoredKernelSystem).unwrap().unwrap().is_none());
        kernel_system_registry.insert(index, resource_id);
    }

    const FINISH_BACKGROUND_PROCESSOR_ORDER: usize = 0;
    const EVENT_MANAGER_ORDER: usize = Self::FINISH_BACKGROUND_PROCESSOR_ORDER + 1;
    const EXECUTABLE_MANAGER_ORDER: usize = Self::EVENT_MANAGER_ORDER + 1;
    const DELAY_MANAGER_ORDER: usize = Self::EXECUTABLE_MANAGER_ORDER + 1;
    const BLOCKER_MANAGER_ORDER: usize = Self::DELAY_MANAGER_ORDER + 1;
    const PROCESSOR_ORDER: usize = Self::BLOCKER_MANAGER_ORDER + 1;
    const CONDITIONAL_EVENTS_ORDER: usize = Self::PROCESSOR_ORDER + 1;
    const START_BACKGROUND_PROCESSOR_ORDER: usize = Self::CONDITIONAL_EVENTS_ORDER + 1;
    pub fn load_default(&self, processor_threads: usize) {
        // Finish & Start have a special relationship so this first part is for that
        let mut finish_background_processor = FinishBackgroundProcessor::default();
        let resource_id = finish_background_processor.init(&self.state);
        let start_background_processor = StartBackgroundProcessor::create_from(&finish_background_processor).unwrap();

        {
            let mut kernel_system_registry = self.state.resolve::<Unique<KernelSystemRegistry>>(Some(&self.program_id), None, None, Some(&self.kernel_key)).unwrap().unwrap();
    
            assert!(self.state.insert(Some(&self.program_id), Some(resource_id.clone()), Some(&self.kernel_key), Box::new(finish_background_processor) as StoredKernelSystem).unwrap().unwrap().is_none());
            kernel_system_registry.insert(Self::FINISH_BACKGROUND_PROCESSOR_ORDER, resource_id);
        }
        //

        self.load_kernel_system(EventManager, Self::EVENT_MANAGER_ORDER);
        self.load_kernel_system(ExecutableManager, Self::EXECUTABLE_MANAGER_ORDER);
        self.load_kernel_system(BlockerManager, Self::BLOCKER_MANAGER_ORDER);
        self.load_kernel_system(DelayManager, Self::DELAY_MANAGER_ORDER);
        self.load_kernel_system(start_background_processor, Self::START_BACKGROUND_PROCESSOR_ORDER);
        self.load_kernel_system(ReadOnlyProcessorManager, Self::CONDITIONAL_EVENTS_ORDER);

        // Refactor: Make a separate unit struct for the `KernelSystem` trait and the rest are on a separate struct the unit instantiates in init
        // So processor can get the processor_threads from memory/state before hand
        let processor = Processor::new(processor_threads);
        self.load_kernel_system(processor, Self::PROCESSOR_ORDER);
    }

    pub fn resolve<T: Injection>(&self, program_id: Option<&Id>, resource_id: Option<&ResourceId>, source: Option<&Source>, key: Option<&Key>) -> Option<Result<T::Item<'_>, ResolveError>> {
        self.state.resolve::<T>(program_id, resource_id, source, key)
    }

    pub fn insert<T: 'static>(&self, program_id: Option<&Id>, resource_id: Option<ResourceId>, key: Option<&Key>, resource: T) -> Option<Result<Option<Resource>, InsertError>> {
        self.state.insert(program_id, resource_id, key, resource)
    }

    pub fn insert_program(&self, program_id: Id, memory_domain: Arc<MemoryDomain>, key: Option<Key>) -> bool {
        self.state.insert_program(program_id, memory_domain, key)
    }

    pub async fn transition(&self) {
        let mut kernel_systems = self.state.resolve::<Unique<KernelSystemRegistry>>(Some(&self.program_id), None, None, Some(&self.kernel_key)).unwrap().unwrap();
        for kernel_systems in kernel_systems.iter() {
            for kernel_system in kernel_systems {
                let mut kernel_system = self.state.resolve::<Unique<StoredKernelSystem>>(Some(&self.program_id), Some(&kernel_system), None, Some(&self.kernel_key)).unwrap().unwrap();
                kernel_system.tick(&self.state).await;
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