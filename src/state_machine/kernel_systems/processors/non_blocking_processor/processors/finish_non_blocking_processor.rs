use std::{pin::Pin, sync::Arc};

use tracing::{Level, event};

use crate::prelude::{AsyncJoinHandles, BackgroundProcessorSystemRegistry, KernelSystem, Memory, NextEvents, ProgramId, ProgramKey, ResourceId, Shared, StartNonBlockingProcessor, StateMachine, StoredSystem, SyncJoinHandles, SystemId, SystemMetadata, Unique};

#[derive(Default)]
pub struct FinishNonBlockingProcessor;

impl FinishNonBlockingProcessor {
    pub fn insert_system(state_machine: &StateMachine, system_id: SystemId, system_metadata: SystemMetadata, stored_system: StoredSystem) -> Option<Option<SystemMetadata>> {
        StartNonBlockingProcessor::insert_system(state_machine, system_id, system_metadata, stored_system)
    }
}

impl KernelSystem for FinishNonBlockingProcessor {
    fn system_id(&self) -> SystemId {
        SystemId::from("Finishing NonBlocking Processor")    
    }

    fn init(&mut self, memory: &Memory, kernel_program_id: &ProgramId, kernel_program_key: &ProgramKey) {
        event!(Level::TRACE, status="Initialising", kernel_system_id = ?self.system_id());
        
        // assert!(matches!(memory.contains_resource(None, &ResourceId::from_raw_heap::<NextEvents>(), None), Some(true)));
        // assert!(matches!(memory.contains_resource(None, &ResourceId::from_raw_heap::<BackgroundProcessorSystemRegistry>(), None), Some(true)));
        
        assert!(memory.insert(Some(&kernel_program_id), None, Some(kernel_program_key), AsyncJoinHandles::default()).unwrap().is_ok());
        assert!(memory.insert(Some(&kernel_program_id), None, Some(kernel_program_key), SyncJoinHandles::default()).unwrap().is_ok());

        event!(Level::TRACE, status="Initialised", kernel_system_id = ?self.system_id());
    }

    fn tick(&mut self, memory: &Arc<Memory>, kernel_program_id: ProgramId, kernel_program_key: ProgramKey) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let mut async_join_handles = memory.resolve::<Unique<AsyncJoinHandles>>(Some(&kernel_program_id), None, None, Some(&kernel_program_key)).unwrap().unwrap();
            let mut sync_join_handles = memory.resolve::<Unique<SyncJoinHandles>>(Some(&kernel_program_id), None, None, Some(&kernel_program_key)).unwrap().unwrap();

            let mut next_events = memory.resolve::<Unique<NextEvents>>(None, None, None, None).unwrap().unwrap();

            let system_registry = memory.resolve::<Shared<BackgroundProcessorSystemRegistry>>(None, None, None, None).unwrap().unwrap();

            let async_finished = async_join_handles.get_finished().await;
            for (id, finished) in async_finished {
                let finished = finished.unwrap();
                
                let resource_id = system_registry.get(&id).unwrap().stored_system_metadata().resource_id();

                let mut system = memory.resolve::<Unique<StoredSystem>>(None, Some(&resource_id), None, None).unwrap().unwrap();

                system.insert_system(finished);
                next_events.insert(id.into_id());
            }

            let sync_finished = sync_join_handles.get_finished();
            for (id, finished) in sync_finished {
                let finished = finished.unwrap();

                let resource_id = system_registry.get(&id).unwrap().stored_system_metadata().resource_id();

                let mut system = memory.resolve::<Unique<StoredSystem>>(None, Some(&resource_id), None, None).unwrap().unwrap();

                system.insert_system(finished);
                next_events.insert(id.into_id());
            }
        })
    }
}