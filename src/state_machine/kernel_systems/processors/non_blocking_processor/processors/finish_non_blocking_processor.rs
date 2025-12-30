use std::{pin::Pin, sync::Arc};

use tracing::{Level, event, span};

use crate::prelude::{AsyncJoinHandles, BackgroundProcessorSystemRegistry, KernelSystem, Memory, NextBlockers, NextEvents, ProgramId, ProgramKey, ResourceId, Shared, StartNonBlockingProcessor, StateMachine, StoredSystem, SyncJoinHandles, SystemEventRegistry, SystemId, SystemMetadata, Unique};

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
        event!(Level::DEBUG, "Inserting AsyncJoinHandles");
        assert!(memory.insert(Some(&kernel_program_id), None, Some(kernel_program_key), AsyncJoinHandles::default()).unwrap().is_ok());
        
        event!(Level::DEBUG, "Inserting SyncJoinHandles");
        assert!(memory.insert(Some(&kernel_program_id), None, Some(kernel_program_key), SyncJoinHandles::default()).unwrap().is_ok());
        
        event!(Level::DEBUG, "Inserting BackgroundProcessorSystemRegistry");
        assert!(memory.insert(None, None, None, BackgroundProcessorSystemRegistry::default()).unwrap().is_ok());
        
        event!(Level::DEBUG, "Checking NextEvents");
        if !matches!(memory.contains_resource(None, &ResourceId::from_raw_heap::<NextEvents>(), None), Some(true)) {
            event!(Level::WARN, "NextEvents Not Found")
        }

        event!(Level::DEBUG, "Checking NextBlockers");
        if !matches!(memory.contains_resource(None, &ResourceId::from_raw_heap::<NextBlockers>(), None), Some(true)) {
            event!(Level::WARN, "NextBlockers Not Found")
        }

        event!(Level::DEBUG, "Checking SystemEventRegistry");
        if !matches!(memory.contains_resource(None, &ResourceId::from_raw_heap::<SystemEventRegistry>(), None), Some(true)) {
            event!(Level::WARN, "SystemEventRegistry Not Found")
        }
    }

    fn tick(&mut self, memory: &Arc<Memory>, kernel_program_id: ProgramId, kernel_program_key: ProgramKey) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let mut async_join_handles = memory.resolve::<Unique<AsyncJoinHandles>>(Some(&kernel_program_id), None, None, Some(&kernel_program_key)).unwrap().unwrap();
            let mut sync_join_handles = memory.resolve::<Unique<SyncJoinHandles>>(Some(&kernel_program_id), None, None, Some(&kernel_program_key)).unwrap().unwrap();

            let system_registry = memory.resolve::<Shared<BackgroundProcessorSystemRegistry>>(None, None, None, None).unwrap().unwrap();
            
            let mut next_events = memory.resolve::<Unique<NextEvents>>(None, None, None, None).unwrap().unwrap();
            let mut next_blockers = memory.resolve::<Unique<NextBlockers>>(None, None, None, None).unwrap().unwrap();

            let system_event_registry = memory.resolve::<Shared<SystemEventRegistry>>(None, None, None, None).unwrap().unwrap();
            
            let async_finished = async_join_handles.get_finished().await;
            
            let finished_span = span!(Level::DEBUG, "Finished Non Blocking");
            let _enter = finished_span;
            
            let async_span = span!(Level::DEBUG, "Async");
            let _enter = async_span.enter();

            event!(Level::DEBUG, non_blocking_async_finished_count=async_finished.len(), "Systems Count");

            for (system_id, finished) in async_finished {
                let system_span = span!(Level::TRACE, "System", system_id=?system_id);
                let _enter = system_span.enter();

                event!(Level::TRACE, "System");

                // todo dont do unwrap
                let (finished, result) = finished.unwrap();
                
                let resource_id = system_registry.get(&system_id).unwrap().stored_system_metadata().resource_id();

                let mut system = memory.resolve::<Unique<StoredSystem>>(None, Some(&resource_id), None, None).unwrap().unwrap();

                system.insert_system(finished);

                if let Some(result) = result {
                    event!(Level::TRACE, result=?result, "System Returned Result");
                    result.act(
                        &system_id,
                        &mut next_events,
                        &mut next_blockers,
                        &system_event_registry,
                        system_span.clone()
                    );
                }
            }
            drop(_enter);

            let sync_finished = sync_join_handles.get_finished();

            let sync_span = span!(Level::DEBUG, "Sync");
            let _enter = sync_span.enter();

            event!(Level::DEBUG, non_blocking_sync_finished_count=sync_finished.len(), "Systems Count");

            for (system_id, finished) in sync_finished {
                let system_span = span!(Level::TRACE, "System", system_id=?system_id);
                let _enter = system_span.enter();

                event!(Level::TRACE, "System");

                // todo dont do unwrap
                let (finished, result) = finished.unwrap();
                
                let resource_id = system_registry.get(&system_id).unwrap().stored_system_metadata().resource_id();
                
                let mut system = memory.resolve::<Unique<StoredSystem>>(None, Some(&resource_id), None, None).unwrap().unwrap();
                
                system.insert_system(finished);

                if let Some(result) = result {
                    event!(Level::TRACE, result=?result, "System Returned Result");
                    result.act(
                        &system_id,
                        &mut next_events,
                        &mut next_blockers,
                        &system_event_registry,
                        system_span.clone()
                    );
                }
            }
        })
    }
}