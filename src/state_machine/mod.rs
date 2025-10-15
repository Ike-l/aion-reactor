use std::sync::Arc;

use threadpool::ThreadPool;

use crate::{id::Id, injection::{injection_primitives::unique::Unique, injection_trait::Injection}, memory::{access_checked_heap::heap::HeapId, errors::ResolveError, resource_id::Resource, Memory, ResourceId}, state_machine::{kernel_registry::KernelSystemRegistry, kernel_systems::{background_processor::BackgroundProcessor, blocker_manager::BlockerManager, event_manager::EventManager, processor::Processor, StoredKernelSystem}}, system::system_metadata::Source};

pub mod kernel_systems;
pub mod kernel_registry;

#[derive(Debug)]
pub struct StateMachine {
    memory: Arc<Memory>,
    threadpool: ThreadPool,
    runtime: Arc<tokio::runtime::Runtime>
}

impl StateMachine {
    pub fn new(threads: usize) -> Self {
        let memory = Arc::new(Memory::new());
        
        memory.insert(
            None, 
            None, 
            KernelSystemRegistry::default()
        );

        Self {
            memory,
            threadpool: ThreadPool::new(threads),
            runtime: Arc::new(tokio::runtime::Runtime::new().unwrap())
        }
    }

    pub fn load_default(&self, processor_threads: usize) {
        let mut kernel_system_registry = self.memory.quick_resolve::<Unique<KernelSystemRegistry>>();

        let event_manager_id = ResourceId::Heap(HeapId::Label(Id("KernelEventManager".to_string())));
        self.memory.insert(None, Some(event_manager_id.clone()), EventManager::new());
        kernel_system_registry.insert(0, event_manager_id);

        let blocker_manager_id = ResourceId::Heap(HeapId::Label(Id("KernelBlockerManager".to_string())));
        self.memory.insert(None, Some(blocker_manager_id.clone()), BlockerManager::new());
        kernel_system_registry.insert(0, blocker_manager_id);

        let processor_resource_id = ResourceId::Heap(HeapId::Label(Id("KernelProcessor".to_string())));
        self.memory.insert(None, Some(processor_resource_id.clone()), Processor::new(processor_threads));
        kernel_system_registry.insert(1, processor_resource_id);

        let background_processor_resource_id = ResourceId::Heap(HeapId::Label(Id("KernelBackgroundProcessor".to_string())));
        self.memory.insert(None, Some(background_processor_resource_id.clone()), BackgroundProcessor::new());
        kernel_system_registry.insert(1, background_processor_resource_id);
    }

    pub fn resolve<T: Injection>(&self, program_id: Option<&Id>, resource_id: Option<&ResourceId>, source: Option<&Source>) -> Option<Result<T::Item<'_>, ResolveError>> {
        self.memory.resolve::<T>(program_id, resource_id, source)
    }

    pub fn insert<T: 'static>(&self, program_id: Option<&Id>, resource_id: Option<ResourceId>, resource: T) -> Option<Option<Resource>> {
        self.memory.insert(program_id, resource_id, resource)
    }

    pub async fn tick(&self) {
        let mut kernel_systems = self.memory.resolve::<Unique<KernelSystemRegistry>>(None, None, None).unwrap().unwrap();
        for kernel_systems in kernel_systems.iter() {
            for kernel_system in kernel_systems.clone() {
                let memory = Arc::clone(&self.memory);
                let runtime = Arc::clone(&self.runtime);
                self.threadpool.execute(move || {
                    let mut kernel_system = memory.resolve::<Unique<StoredKernelSystem>>(None, Some(&kernel_system), None).unwrap().unwrap();
                    runtime.block_on(kernel_system.tick(&memory));
                });
            }

            self.threadpool.join();
        }
    }
}

#[cfg(test)]
mod state_machine_tests {
    // test tick
    // test insert resource + [conflict/no conflict]
    // test get resource + [exist/no exist]
}