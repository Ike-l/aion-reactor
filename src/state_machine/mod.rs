use std::{collections::HashSet, sync::{Arc, Mutex}};

use threadpool::ThreadPool;

use crate::{id::Id, injection::{injection_primitives::unique::Unique, injection_trait::Injection, AccessDropper}, memory::{access_checked_heap::heap::HeapId, errors::ResolveError, resource_id::Resource, Memory, ResourceId}, state_machine::{blacklist::Blacklist, kernel_registry::KernelSystemRegistry, kernel_systems::{background_processor::BackgroundProcessor, blocker_manager::BlockerManager, event_manager::{EventManager, EventMapper}, processor::Processor, StoredKernelSystem}, transition_phases::TransitionPhase}, system::{stored_system::StoredSystem, system_metadata::{Source, SystemMetadata}}};

pub mod kernel_systems;
pub mod kernel_registry;
pub mod blacklist;
pub mod transition_phases;

#[derive(Debug)]
pub struct StateMachine {
    state: Arc<Memory>,
    threadpool: ThreadPool,
    runtime: Arc<tokio::runtime::Runtime>,

    keys: Arc<Mutex<HashSet<u64>>>
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
            state: memory,
            threadpool: ThreadPool::new(threads),
            runtime: Arc::new(tokio::runtime::Runtime::new().unwrap()),
            keys: Arc::new(Mutex::new(HashSet::new()))
        }
    }

    pub fn load_default(&self, processor_threads: usize) {
        let blacklist_resource_id = ResourceId::from(HeapId::from(Id("Blacklist".to_string())));

        let mut kernel_system_registry = self.state.quick_resolve::<Unique<KernelSystemRegistry>>();

        let event_manager_id = ResourceId::Heap(HeapId::Label(Id("KernelEventManager".to_string())));
        self.state.insert(None, Some(event_manager_id.clone()), EventManager::new(&self.state));
        kernel_system_registry.insert(0, event_manager_id.clone());

        let blocker_manager_id = ResourceId::Heap(HeapId::Label(Id("KernelBlockerManager".to_string())));
        self.state.insert(None, Some(blocker_manager_id.clone()), BlockerManager::new());
        kernel_system_registry.insert(0, blocker_manager_id.clone());
        
        let background_processor_resource_id = ResourceId::Heap(HeapId::Label(Id("KernelBackgroundProcessor".to_string())));
        self.state.insert(None, Some(background_processor_resource_id.clone()), BackgroundProcessor::new());
        kernel_system_registry.insert(1, background_processor_resource_id.clone());
        
        let processor_resource_id = ResourceId::Heap(HeapId::Label(Id("KernelProcessor".to_string())));
        self.state.insert(None, Some(processor_resource_id.clone()), Processor::new(processor_threads));
        kernel_system_registry.insert(1, processor_resource_id.clone());
        
        
        let mut blacklist = Blacklist::new();
        
        let block_keys = Arc::clone(&self.keys);
        let unblock_keys = Arc::clone(&self.keys);
        blacklist.insert_block(
            move |memory| {
                let a = memory.resolve::<Unique<EventManager>>(None, Some(&event_manager_id), None).unwrap().unwrap();
                let e = memory.resolve::<Unique<EventMapper>>(None, None, None).unwrap().unwrap();
                
                let b = memory.resolve::<Unique<BlockerManager>>(None, Some(&blocker_manager_id), None).unwrap().unwrap();
                
                let c = memory.resolve::<Unique<BackgroundProcessor>>(None, Some(&background_processor_resource_id), None).unwrap().unwrap();
                let d = memory.resolve::<Unique<Processor>>(None, Some(&processor_resource_id), None).unwrap().unwrap();

                let mut keys = block_keys.lock().unwrap();
                keys.extend(vec![
                    a.access_dropper().delay_dropper(),
                    b.access_dropper().delay_dropper(),
                    c.access_dropper().delay_dropper(),
                    d.access_dropper().delay_dropper(),
                    e.access_dropper().delay_dropper(),
                ]);

        }, move |memory| {
            let mut keys = unblock_keys.lock().unwrap();
            for key in keys.drain() {
                memory.end_drop_delay(key, None).unwrap();
            }
        });
            
        self.state.insert(None, Some(blacklist_resource_id), blacklist);
            
    }

    pub fn resolve<T: Injection>(&self, program_id: Option<&Id>, resource_id: Option<&ResourceId>, source: Option<&Source>) -> Option<Result<T::Item<'_>, ResolveError>> {
        self.state.resolve::<T>(program_id, resource_id, source)
    }

    pub fn insert<T: 'static>(&self, program_id: Option<&Id>, resource_id: Option<ResourceId>, resource: T) -> Option<Option<Resource>> {
        self.state.insert(program_id, resource_id, resource)
    }

    pub async fn transition(&self) {
        let mut kernel_systems = self.state.resolve::<Unique<KernelSystemRegistry>>(None, None, None).unwrap().unwrap();
        for phase in TransitionPhase::iter_fields() {
            for kernel_systems in kernel_systems.iter() {
                for kernel_system in kernel_systems.clone() {
                    let memory = Arc::clone(&self.state);
                    let runtime = Arc::clone(&self.runtime);
                    self.threadpool.execute(move || {
                        let mut kernel_system = memory.resolve::<Unique<StoredKernelSystem>>(None, Some(&kernel_system), None).unwrap().unwrap();
                        runtime.block_on(kernel_system.tick(&memory, phase));
                    });
                }
    
                self.threadpool.join();
            }
        }
    }

    pub fn insert_system(&self, id: Id, system_metadata: SystemMetadata, system: StoredSystem) -> Option<SystemMetadata> {
        Processor::insert_system(&self.state, id, system_metadata, system)
    }
}

#[cfg(test)]
mod state_machine_tests {
    // test tick
    // test insert resource + [conflict/no conflict]
    // test get resource + [exist/no exist]
}