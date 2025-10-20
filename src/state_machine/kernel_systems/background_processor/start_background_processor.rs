use std::{pin::Pin, sync::Arc};

use crate::{injection::injection_primitives::{shared::Shared, unique::Unique}, memory::Memory, state_machine::{kernel_systems::{background_processor::background_processor_system_registry::BackgroundProcessorSystemRegistry, processor::Processor, KernelSystem}, transition_phases::TransitionPhase}, system::{stored_system::StoredSystem, system_metadata::Source, System}};

pub struct StartBackgroundProcessor;

impl StartBackgroundProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl KernelSystem for StartBackgroundProcessor {
    fn tick(&mut self, memory: &Arc<Memory>, _phase: TransitionPhase) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let system_registry = memory.resolve::<Shared<BackgroundProcessorSystemRegistry>>(None, None, None).unwrap().unwrap();
            let systems = Processor::get_systems(&memory, &system_registry.0);

            if systems.iter().any(|(&id, system_metadata)| {
                let program_id = system_metadata.program_id();
                let resource_id = system_metadata.resource_id();
    
                let system = memory.resolve::<Shared<StoredSystem>>(program_id.as_ref(), Some(resource_id), None).unwrap().unwrap();
                match system.reserve_accesses(&memory, program_id.as_ref(), Source(id.clone())) {
                    Some(true) => {
                        true
                    }
                    None | Some(false) => {
                        false
                    }
                }
                }) {
                panic!("Conflicting accesses")
            }

            let mut new_async_join_handles = Vec::new();
            let mut new_sync_join_handles = Vec::new();
            for (id, system_metadata) in systems {
                let program_id = system_metadata.program_id();
                let resource_id = system_metadata.resource_id();
    
                let mut system = memory.resolve::<Unique<StoredSystem>>(program_id.as_ref(), Some(resource_id), None).unwrap().unwrap();

                let source = Source(id.clone());

                if let Some(system) = system.take_system() {
                    match system {
                        System::Sync(mut sync_system) => {
                            let memory_clone = Arc::clone(&memory);
                            let program_id = program_id.clone();
                            let source = source.clone();
                            let join_handle = std::thread::spawn(move || {
                                sync_system.run(&memory_clone, program_id.as_ref(), Some(&source));
                                System::Sync(sync_system)
                            });

                            new_sync_join_handles.push(join_handle);
                        }
                        System::Async(mut async_system) => {
                            let memory_clone = Arc::clone(&memory);
                            let program_id = program_id.clone();
                            let source = source.clone();
                            let join_handle = tokio::spawn(async move {
                                async_system.run(memory_clone, program_id, Some(source)).await;
                                System::Async(async_system)
                            });

                            new_async_join_handles.push(join_handle);
                        }
                    }
                } else {
                    panic!("UB");
                }
            }
        })
    }
}