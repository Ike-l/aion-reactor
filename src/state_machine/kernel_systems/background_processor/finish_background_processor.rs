use std::{pin::Pin, sync::Arc};

use crate::{id::Id, injection::injection_primitives::{shared::Shared, unique::Unique}, memory::{access_checked_heap::heap::HeapId, Memory, ResourceId}, state_machine::{kernel_systems::{background_processor::{async_join_handles::AsyncJoinHandles, background_processor_system_registry::BackgroundProcessorSystemRegistry, sync_join_handles::SyncJoinHandles}, event_manager::event::NextEvents, KernelSystem, StoredKernelSystem}, transition_phases::TransitionPhase}, system::stored_system::StoredSystem};

pub struct FinishBackgroundProcessor;

impl KernelSystem for FinishBackgroundProcessor {
    fn init(&mut self, memory: &Memory) -> ResourceId {
        memory.insert(None, None, AsyncJoinHandles::default()).unwrap();
        memory.insert(None, None, SyncJoinHandles::default()).unwrap();

        let finish_background_processor_resource_id = ResourceId::Heap(HeapId::Label(Id("KernelFinishBackgroundProcessor".to_string())));
        memory.insert(None, Some(finish_background_processor_resource_id.clone()), Box::new(Self) as StoredKernelSystem);
        finish_background_processor_resource_id
    }

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