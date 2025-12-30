use std::{pin::Pin, sync::Arc};

use tracing::{Level, event, span};

use crate::prelude::{AsyncJoinHandles, BackgroundProcessorSystemRegistry, KernelSystem, Memory, NextEvents, Processor, ProgramId, ProgramKey, ResourceId, Shared, StateMachine, StoredSystem, SyncJoinHandles, SystemId, SystemMetadata, Unique};

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
        event!(Level::DEBUG, "Checking Arc<tokio::runtime::Runtime>");
        if !matches!(memory.contains_resource(Some(kernel_program_id), &ResourceId::from_raw_heap::<Arc<tokio::runtime::Runtime>>(), Some(kernel_program_key)), Some(true)) {
            event!(Level::WARN, "Arc<tokio::runtime::Runtime> Not Found")
        }
        
        event!(Level::DEBUG, "Checking AsyncJoinHandles");
        if !matches!(memory.contains_resource(Some(kernel_program_id), &ResourceId::from_raw_heap::<AsyncJoinHandles>(), Some(kernel_program_key)), Some(true)) {
            event!(Level::WARN, "AsyncJoinHandles Not Found")
        }
        
        event!(Level::DEBUG, "Checking SyncJoinHandles");
        if !matches!(memory.contains_resource(Some(kernel_program_id), &ResourceId::from_raw_heap::<SyncJoinHandles>(), Some(kernel_program_key)), Some(true)) {
            event!(Level::WARN, "SyncJoinHandles Not Found")
        }
        
        event!(Level::DEBUG, "Checking BackgroundProcessorSystemRegistry");
        if !matches!(memory.contains_resource(None, &ResourceId::from_raw_heap::<BackgroundProcessorSystemRegistry>(), None), Some(true)) {
            event!(Level::WARN, "BackgroundProcessorSystemRegistry Not Found")
        }

        event!(Level::DEBUG, "Checking NextEvents");
        if !matches!(memory.contains_resource(None, &ResourceId::from_raw_heap::<NextEvents>(), None), Some(true)) {
            event!(Level::WARN, "NextEvents Not Found")
        }
    }

    fn tick(&mut self, memory: &Arc<Memory>, kernel_program_id: ProgramId, kernel_program_key: ProgramKey) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let system_registry = memory.resolve::<Shared<BackgroundProcessorSystemRegistry>>(None, None, None, None).unwrap().unwrap();
            
            let systems = Processor::get_systems(&memory, system_registry.ref_generic());

            {
                let span = span!(Level::TRACE, "System Derived Events");
                let _enter = span.enter();
                let mut next_events = memory.resolve::<Unique<NextEvents>>(None, None, None, None).unwrap().unwrap();
                for &id in systems.keys() {
                    let event_id = id.clone().into_id();
                    event!(Level::TRACE, event=?event_id, "New Event");
                    next_events.insert(event_id);
                }
            }

            event!(Level::DEBUG, "Executing");

            let runtime = memory.resolve::<Shared<Arc<tokio::runtime::Runtime>>>(
                Some(&kernel_program_id), 
                None, 
                None, 
                Some(&kernel_program_key)
            ).unwrap().unwrap();

            let (
                new_async_join_handles, 
                new_sync_join_handles
            ) = Processor::execute_non_blocking(
                &memory,
                systems,
                &runtime
            ).await;

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