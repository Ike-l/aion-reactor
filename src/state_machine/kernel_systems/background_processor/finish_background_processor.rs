use std::{pin::Pin, sync::Arc};

use crate::{id::Id, injection::injection_primitives::{shared::Shared, unique::Unique}, memory::{access_checked_heap::heap::HeapId, memory_domain::MemoryDomain, program_memory_map::inner_program_memory_map::Key, Memory, ResourceId}, state_machine::{kernel_systems::{background_processor::{async_join_handles::AsyncJoinHandles, background_processor_system_registry::BackgroundProcessorSystemRegistry, start_background_processor::StartBackgroundProcessor, sync_join_handles::SyncJoinHandles}, event_manager::event::NextEvents, KernelSystem}, transition_phases::TransitionPhase}, system::stored_system::StoredSystem};

#[derive(Default)]
pub struct FinishBackgroundProcessor {
    program_id: Option<Id>,
    key: Option<Key>,
}

impl FinishBackgroundProcessor {
    pub fn create_starter(&self) -> Option<StartBackgroundProcessor> {
        Some(StartBackgroundProcessor::new(self.program_id.as_ref()?.clone(), self.key.as_ref()?.clone()))
    }
}

impl KernelSystem for FinishBackgroundProcessor {
    fn init(&mut self, memory: &Memory) -> ResourceId {
        let program_id = Id("_FinishBackgroundProcessor".to_string());
        
        let key = Some(rand::random());
        memory.insert_program(program_id.clone(), Arc::new(MemoryDomain::new()), key.clone());

        assert!(memory.insert(Some(&program_id), None, key.as_ref(), AsyncJoinHandles::default()).unwrap().is_ok());
        assert!(memory.insert(Some(&program_id), None, key.as_ref(), SyncJoinHandles::default()).unwrap().is_ok());

        self.key = key;
        self.program_id.replace(program_id);

        ResourceId::Heap(HeapId::Label(Id("KernelFinishBackgroundProcessor".to_string())))
    }

    fn tick(&mut self, memory: &Arc<Memory>, _phase: TransitionPhase) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let mut async_join_handles = memory.resolve::<Unique<AsyncJoinHandles>>(self.program_id.as_ref(), None, None, self.key.as_ref()).unwrap().unwrap();
            let mut sync_join_handles = memory.resolve::<Unique<SyncJoinHandles>>(self.program_id.as_ref(), None, None, self.key.as_ref()).unwrap().unwrap();

            let mut next_events = memory.resolve::<Unique<NextEvents>>(None, None, None, None).unwrap().unwrap();

            let system_registry = memory.resolve::<Shared<BackgroundProcessorSystemRegistry>>(None, None, None, None).unwrap().unwrap();

            let async_finished = async_join_handles.get_finished().await;
            for (id, finished) in async_finished {
                let finished = finished.unwrap();
                
                let resource_id = system_registry.0.get(&id).unwrap().resource_id();

                let mut system = memory.resolve::<Unique<StoredSystem>>(None, Some(&resource_id), None, None).unwrap().unwrap();

                system.insert_system(finished);
                next_events.insert(id);
            }

            let sync_finished = sync_join_handles.get_finished();
            for (id, finished) in sync_finished {
                let finished = finished.unwrap();

                let resource_id = system_registry.0.get(&id).unwrap().resource_id();

                let mut system = memory.resolve::<Unique<StoredSystem>>(None, Some(&resource_id), None, None).unwrap().unwrap();

                system.insert_system(finished);
                next_events.insert(id);
            }
        })
    }
}