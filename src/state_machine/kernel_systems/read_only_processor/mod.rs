use std::{collections::HashMap, pin::Pin, sync::Arc};

use threadpool::ThreadPool;

use crate::{id::Id, injection::{injection_primitives::{cloned::Cloned, shared::Shared, unique::Unique}, injection_trait::Injection}, kernel_prelude::KernelSystem, memory::{Memory, ResourceId, access_checked_heap::heap::HeapId}, state_machine::kernel_systems::{event_manager::event::{Event, NextEvents}, processor::Processor}, system::{System, stored_system::StoredSystem, sync_system::SyncSystem, system_cell::SystemCell, system_metadata::{Source, SystemRegistry}, system_result::SystemResult}};

pub struct ReadOnlyProcessor {
    threadpool: ThreadPool
}

impl ReadOnlyProcessor {
    pub fn new(num_threads: usize) -> Self {
        Self {
            threadpool: ThreadPool::new(num_threads)
        }
    }

    pub async fn execute(&self, systems: Vec<Id>, memory: &Arc<Memory>) -> Vec<(Id, SystemResult)> {
        let threads = self.threadpool.max_count();
        let chunk_size = systems.len() / threads;
        let systems = Arc::new(systems);

        let system_map = Arc::new(
            memory.resolve::<Shared<ReadOnlySystemRegistry>>(None, None, None, None)
                .unwrap().unwrap().0
                .into_map()
                .collect::<HashMap<_, _>>()
        );

        let system_mapping: Arc<HashMap<Id, SystemCell>> = Arc::new(
            system_map.iter().map(|(id, (resource_id, program_id, key))| {
                (id.clone(), SystemCell::new(memory.resolve::<Unique<StoredSystem>>(program_id.as_ref(), Some(resource_id), None, key.as_ref()).unwrap().unwrap().take_system().unwrap()))
            }).collect() 
        );

        let (results_tx, mut results_rx) = tokio::sync::mpsc::unbounded_channel();

        for thread in 0..threads {
            let chunk = thread * chunk_size;
            
            let memory = Arc::clone(&memory);
            let systems = Arc::clone(&systems);   
            let system_mapping = Arc::clone(&system_mapping);             
            let system_map = Arc::clone(&system_map);
            let results_tx = results_tx.clone();
            
            self.threadpool.execute(move || {
                for i in chunk..((chunk + chunk_size).min(systems.len())) {
                    let id = &systems[i];

                    // Safety:
                    // The same system is not accessed any where else
                    let inner = unsafe {
                        system_mapping.get(&id).unwrap().get()
                    };

                    let (_, program_id, key) = system_map.get(&id).unwrap();
                    let source = Source(id.clone());


                    match inner {
                        System::Sync(sync_system) => {
                            if !sync_system.check_read_only(Some(&source)) {
                                continue;
                            }

                            if let Some(result) = sync_system.run(
                                &memory, 
                                program_id.as_ref(), 
                                Some(&source), 
                                key.as_ref()
                            ) {
                                let _ = results_tx.send((id.clone(), result));
                            }
                        },
                        // spawn on tokio?
                        System::Async(_async_system) => todo!(),
                    }
                }
            });
        }
        
        drop(results_tx);

        // todo!("Catch panics: using results_tx?");
        self.threadpool.join();
        
        results_rx.close();
        
        let mut system_mapping = Arc::try_unwrap(system_mapping).unwrap();
        for (id, mut stored_system) in system_map.iter().map(|(id, (resource_id, program_id, key))| {
                (id, memory.resolve::<Unique<StoredSystem>>(program_id.as_ref(), Some(resource_id), None, key.as_ref()).unwrap().unwrap())
        }) {
            stored_system.insert_system(system_mapping.remove(id).unwrap().consume());
        }


        let mut results = Vec::new();

        // non blocking?
        results_rx.recv_many(&mut results, systems.len()).await;

        results
    }
}

#[derive(Default)]
pub struct ReadOnlySystemRegistry(pub SystemRegistry);

#[derive(Default)]
pub struct SystemEventRegistry(pub HashMap<Id, Vec<Event>>);

const SYSTEM_EVENT_REGISTRY_TYPE_NAME: &'static str = "SystemEventRegistry";

impl KernelSystem for ReadOnlyProcessor {
    fn init(&mut self, memory: &Memory) -> ResourceId {
        assert!(memory.insert(None, None, None, ReadOnlySystemRegistry::default()).unwrap().is_ok());
        assert!(memory.insert(None, None, None, SystemEventRegistry::default()).unwrap().is_ok());

        ResourceId::Heap(HeapId::Label(Id("KernelReadOnlyProcessorManager".to_string())))
    }
    
    fn tick(&mut self, memory: &Arc<Memory>) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let system_registry = memory.resolve::<Shared<ReadOnlySystemRegistry>>(None, None, None, None).unwrap().unwrap();
            let systems = Processor::get_systems(&memory, &system_registry.0);
            
            let systems = systems.into_iter().map(|(id, _)| id.clone()).collect::<Vec<_>>();
            
            let results = self.execute(systems, &memory).await;

            let system_event_registry = memory.resolve::<Shared<SystemEventRegistry>>(None, None, None, None).unwrap().unwrap();
            let mut next_events = memory.resolve::<Unique<NextEvents>>(None, None, None, None).unwrap().unwrap();

            for (id, result) in results {
                if matches!(result, SystemResult::Conditional(true)) {
                    if let Some(events) = system_event_registry.0.get(&id) {
                        next_events.extend(events.clone().into_iter());
                    } else {
                        println!("Warn: `ReadOnlySystem` returned `SystemResult::Conditional(true)` without an `Event` mapping\nSuggestion: insert `{SYSTEM_EVENT_REGISTRY_TYPE_NAME}` with system `Id` (`Source`), with the list of `Event`s you want to spawn in `NextEvents`");
                    }
                }
            }
        })
    }
}

pub trait ReadOnlyInjection: Injection {}

impl<T: 'static> ReadOnlyInjection for Shared<'_, T> {}
impl<T: Clone + 'static> ReadOnlyInjection for Cloned<T> {}

pub trait ReadOnlySystem {
    fn check_read_only(&self, source: Option<&Source>) -> bool;    
}

// do AsyncSystems by abstracting SyncSystem & AsyncSystem traits
impl<T: SyncSystem> ReadOnlySystem for T {
    fn check_read_only(&self, source: Option<&Source>) -> bool {
        SyncSystem::check_read_only(self, source)
    }
}
