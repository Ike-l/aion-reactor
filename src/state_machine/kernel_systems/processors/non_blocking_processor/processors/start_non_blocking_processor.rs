use std::{pin::Pin, sync::Arc};

use tracing::{Level, event};

use crate::prelude::{AsyncJoinHandles, BackgroundProcessorSystemRegistry, KernelSystem, Memory, Processor, ProgramId, ProgramKey, ResourceId, Shared, StateMachine, StoredSystem, SyncJoinHandles, System, SystemId, SystemMetadata, Unique};

pub struct StartNonBlockingProcessor;

impl StartNonBlockingProcessor {
    pub fn insert_system(state_machine: &StateMachine, system_id: SystemId, system_metadata: SystemMetadata, stored_system: StoredSystem) -> Option<Option<SystemMetadata>> {
        let mut system_registry = state_machine.memory.resolve::<Unique<BackgroundProcessorSystemRegistry>>(None, None, None, None)?.ok()?;
        Processor::insert_system(state_machine, system_registry.ref_mut_generic(), system_id, system_metadata, stored_system)
    }
}

impl KernelSystem for StartNonBlockingProcessor {
    fn system_id(&self) -> SystemId {
        SystemId::from("Starting NonBlocking Processor")    
    }

    fn init(&mut self, memory: &Memory, kernel_program_id: &ProgramId, kernel_program_key: &ProgramKey) {
        event!(Level::TRACE, status="Initialising", kernel_system_id = ?self.system_id());
        
        assert!(matches!(memory.contains_resource(Some(kernel_program_id), &ResourceId::from_raw_heap::<AsyncJoinHandles>(), Some(kernel_program_key)), Some(true)));
        assert!(matches!(memory.contains_resource(Some(kernel_program_id), &ResourceId::from_raw_heap::<SyncJoinHandles>(), Some(kernel_program_key)), Some(true)));
        
        assert!(memory.insert(None, None, None, BackgroundProcessorSystemRegistry::default()).unwrap().is_ok());
        
        event!(Level::TRACE, status="Initialised", kernel_system_id = ?self.system_id());
    }

    fn tick(&mut self, memory: &Arc<Memory>, kernel_program_id: ProgramId, kernel_program_key: ProgramKey) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let system_registry = memory.resolve::<Shared<BackgroundProcessorSystemRegistry>>(None, None, None, None).unwrap().unwrap();
            
            let systems = Processor::get_systems(self.system_id(), &memory, system_registry.ref_generic());
            
            if systems.iter().any(|(&id, system_metadata)| {
                let program_id = system_metadata.stored_system_metadata().program_id();
                let resource_id = system_metadata.stored_system_metadata().resource_id();
                let key = system_metadata.stored_system_metadata().key();
    
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
                let program_id = system_metadata.stored_system_metadata().program_id();
                let resource_id = system_metadata.stored_system_metadata().resource_id();
    
                let mut system = memory.resolve::<Unique<StoredSystem>>(program_id.as_ref(), Some(resource_id), None, None).unwrap().unwrap();

                let source = id.clone();

                if let Some(system) = system.take_system() {
                    match system {
                        System::Sync(mut sync_system) => {
                            let memory_clone = Arc::clone(&memory);
                            let program_id = program_id.clone();
                            let source = source.clone();
                            let key = system_metadata.stored_system_metadata().key().clone();
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
                            let key = system_metadata.stored_system_metadata().key().clone();
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

            let mut async_join_handles = memory.resolve::<Unique<AsyncJoinHandles>>(Some(&kernel_program_id), None, None, Some(&kernel_program_key)).unwrap().unwrap();
            let mut sync_join_handles = memory.resolve::<Unique<SyncJoinHandles>>(Some(&kernel_program_id), None, None, Some(&kernel_program_key)).unwrap().unwrap();

            for (id, new_async_join_handle) in new_async_join_handles {
                async_join_handles.push(id, new_async_join_handle);
            }
            
            for (id, new_sync_join_handle) in new_sync_join_handles {
                sync_join_handles.push(id, new_sync_join_handle);
            }
        })
    }
}