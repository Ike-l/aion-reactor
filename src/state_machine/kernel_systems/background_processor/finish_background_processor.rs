use std::{pin::Pin, sync::Arc};

use crate::{injection::injection_primitives::{shared::Shared, unique::Unique}, memory::Memory, state_machine::{kernel_systems::{background_processor::{async_join_handles::AsyncJoinHandles, background_processor_system_registry::BackgroundProcessorSystemRegistry, sync_join_handles::SyncJoinHandles}, event_manager::event::NextEvents, KernelSystem}, transition_phases::TransitionPhase}, system::stored_system::StoredSystem};

pub struct FinishBackgroundProcessor;

impl FinishBackgroundProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl KernelSystem for FinishBackgroundProcessor {
    fn tick(&mut self, memory: &Arc<Memory>, _phase: TransitionPhase) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let mut async_join_handles = memory.resolve::<Unique<AsyncJoinHandles>>(None, None, None).unwrap().unwrap();
            let mut sync_join_handles = memory.resolve::<Unique<SyncJoinHandles>>(None, None, None).unwrap().unwrap();

            let mut next_events = memory.resolve::<Unique<NextEvents>>(None, None, None).unwrap().unwrap();

            let system_registry = memory.resolve::<Shared<BackgroundProcessorSystemRegistry>>(None, None, None).unwrap().unwrap();

            let async_finished = async_join_handles.get_finished().await;
            for (id, finished) in async_finished {
                let finished = finished.unwrap();
                
                let resource_id = system_registry.0.get(&id).unwrap().resource_id();

                let mut system = memory.resolve::<Unique<StoredSystem>>(None, Some(&resource_id), None).unwrap().unwrap();

                system.insert_system(finished);
                next_events.insert(id);
            }

            let sync_finished = sync_join_handles.get_finished();
            for (id, finished) in sync_finished {
                let finished = finished.unwrap();

                let resource_id = system_registry.0.get(&id).unwrap().resource_id();

                let mut system = memory.resolve::<Unique<StoredSystem>>(None, Some(&resource_id), None).unwrap().unwrap();

                system.insert_system(finished);
                next_events.insert(id);
            }
        })
    }
}