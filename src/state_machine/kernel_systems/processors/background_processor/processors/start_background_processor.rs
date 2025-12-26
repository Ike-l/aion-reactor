use std::{pin::Pin, sync::Arc};

use crate::prelude::{AsyncJoinHandles, BackgroundProcessorSystemRegistry, FinishBackgroundProcessor, KernelSystem, ProgramKey, Memory, Processor, ProgramId, ResourceId, Shared, StateMachine, StoredSystem, SyncJoinHandles, System, SystemId, SystemMetadata, Unique};

pub struct StartBackgroundProcessor {
    program_id: ProgramId,
    key: ProgramKey
}

impl StartBackgroundProcessor {
    pub fn create_from(finish_background_processor: &FinishBackgroundProcessor) -> Option<Self> {
        let program_id = finish_background_processor.program_id().as_ref()?.clone();
        let key = finish_background_processor.key().as_ref()?.clone();
        
        Some(Self {
            program_id, key
        })
    }

    pub fn insert_system(state_machine: &StateMachine, system_id: SystemId, system_metadata: SystemMetadata, stored_system: StoredSystem) -> Option<Option<SystemMetadata>> {
        let mut system_registry = state_machine.state.resolve::<Unique<BackgroundProcessorSystemRegistry>>(None, None, None, None)?.ok()?;
        Processor::insert_system2(state_machine, system_registry.ref_mut_generic(), system_id, system_metadata, stored_system)
    }
}

impl KernelSystem for StartBackgroundProcessor {
    fn init(&mut self, memory: &Memory) -> ResourceId {
        matches!(memory.contains_resource(None, &ResourceId::from_raw_heap::<AsyncJoinHandles>(), None), Some(true));
        matches!(memory.contains_resource(None, &ResourceId::from_raw_heap::<SyncJoinHandles>(), None), Some(true));
        
        assert!(memory.insert(None, None, None, BackgroundProcessorSystemRegistry::default()).unwrap().is_ok());

        ResourceId::from_labelled_heap("KernelStartBackgroundProcessor")
    }

    fn tick(&mut self, memory: &Arc<Memory>, ) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let system_registry = memory.resolve::<Shared<BackgroundProcessorSystemRegistry>>(None, None, None, None).unwrap().unwrap();
            
            let systems = Processor::get_systems(&memory, system_registry.ref_generic());
            
            if systems.iter().any(|(&id, system_metadata)| {
                let program_id = system_metadata.program_id();
                let resource_id = system_metadata.resource_id();
                let key = system_metadata.key();
    
                let system = memory.resolve::<Shared<StoredSystem>>(program_id.as_ref(), Some(resource_id), None, None).unwrap().unwrap();
                match system.reserve_accesses(&memory, program_id.as_ref(), id.clone(), key.as_ref()) {
                    Ok(Some(Ok(()))) => false,
                    Err(_) => unreachable!(),
                    _ => true
                }
                }) {
                panic!("Conflicting accesses")
            }


            let mut new_async_join_handles = Vec::new();
            let mut new_sync_join_handles = Vec::new();
            for (id, system_metadata) in systems {
                let program_id = system_metadata.program_id();
                let resource_id = system_metadata.resource_id();
    
                let mut system = memory.resolve::<Unique<StoredSystem>>(program_id.as_ref(), Some(resource_id), None, None).unwrap().unwrap();

                let source = id.clone();

                if let Some(system) = system.take_system() {
                    match system {
                        System::Sync(mut sync_system) => {
                            let memory_clone = Arc::clone(&memory);
                            let program_id = program_id.clone();
                            let source = source.clone();
                            let key = system_metadata.key().clone();
                            let join_handle = std::thread::spawn(move || {
                                sync_system.run(&memory_clone, program_id.as_ref(), Some(&source), key.as_ref());
                                System::Sync(sync_system)
                            });

                            new_sync_join_handles.push((id.clone(), join_handle));
                        }
                        System::Async(mut async_system) => {
                            let memory_clone = Arc::clone(&memory);
                            let program_id = program_id.clone();
                            let source = source.clone();
                            let key = system_metadata.key().clone();
                            let join_handle = tokio::spawn(async move {
                                async_system.run(memory_clone, program_id, Some(source), key).await;
                                System::Async(async_system)
                            });

                            new_async_join_handles.push((id.clone(), join_handle));
                        }
                    }
                } else {
                    panic!("UB");
                }
            }

            let mut async_join_handles = memory.resolve::<Unique<AsyncJoinHandles>>(Some(&self.program_id), None, None, Some(&self.key)).unwrap().unwrap();
            let mut sync_join_handles = memory.resolve::<Unique<SyncJoinHandles>>(Some(&self.program_id), None, None, Some(&self.key)).unwrap().unwrap();

            for (id, new_async_join_handle) in new_async_join_handles {
                async_join_handles.push(id, new_async_join_handle);
            }
            
            for (id, new_sync_join_handle) in new_sync_join_handles {
                sync_join_handles.push(id, new_sync_join_handle);
            }
        })
    }
}