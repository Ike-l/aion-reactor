use std::{pin::Pin, sync::Arc};

use crate::prelude::{AsyncJoinHandles, BackgroundProcessorSystemRegistry, KernelSystem, ProgramKey, Memory, MemoryDomain, NextEvents, ProgramId, ResourceId, Shared, StoredSystem, SyncJoinHandles, Unique};

#[derive(Default)]
pub struct FinishBackgroundProcessor {
    program_id: Option<ProgramId>,
    key: Option<ProgramKey>,
}

impl FinishBackgroundProcessor {
    pub fn program_id(&self) -> &Option<ProgramId> {
        &self.program_id
    }

    pub fn key(&self) -> &Option<ProgramKey> {
        &self.key
    }
}

impl KernelSystem for FinishBackgroundProcessor {
    fn init(&mut self, memory: &Memory) -> ResourceId {
        matches!(memory.contains_resource(None, &ResourceId::from_raw_heap::<NextEvents>(), None), Some(true));
        matches!(memory.contains_resource(None, &ResourceId::from_raw_heap::<BackgroundProcessorSystemRegistry>(), None), Some(true));

        let program_id = ProgramId::from("_FinishBackgroundProcessor");
        
        let key = Some(rand::random());
        memory.insert_program(program_id.clone(), Arc::new(MemoryDomain::new()), key.clone());

        assert!(memory.insert(Some(&program_id), None, key.as_ref(), AsyncJoinHandles::default()).unwrap().is_ok());
        assert!(memory.insert(Some(&program_id), None, key.as_ref(), SyncJoinHandles::default()).unwrap().is_ok());

        self.key = key;
        self.program_id.replace(program_id);

        ResourceId::from_labelled_heap("KernelFinishBackgroundProcessor")
    }

    fn tick(&mut self, memory: &Arc<Memory>) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let mut async_join_handles = memory.resolve::<Unique<AsyncJoinHandles>>(self.program_id.as_ref(), None, None, self.key.as_ref()).unwrap().unwrap();
            let mut sync_join_handles = memory.resolve::<Unique<SyncJoinHandles>>(self.program_id.as_ref(), None, None, self.key.as_ref()).unwrap().unwrap();

            let mut next_events = memory.resolve::<Unique<NextEvents>>(None, None, None, None).unwrap().unwrap();

            let system_registry = memory.resolve::<Shared<BackgroundProcessorSystemRegistry>>(None, None, None, None).unwrap().unwrap();

            let async_finished = async_join_handles.get_finished().await;
            for (id, finished) in async_finished {
                let finished = finished.unwrap();
                
                let resource_id = system_registry.get(&id).unwrap().resource_id();

                let mut system = memory.resolve::<Unique<StoredSystem>>(None, Some(&resource_id), None, None).unwrap().unwrap();

                system.insert_system(finished);
                next_events.insert(id.into_id());
            }

            let sync_finished = sync_join_handles.get_finished();
            for (id, finished) in sync_finished {
                let finished = finished.unwrap();

                let resource_id = system_registry.get(&id).unwrap().resource_id();

                let mut system = memory.resolve::<Unique<StoredSystem>>(None, Some(&resource_id), None, None).unwrap().unwrap();

                system.insert_system(finished);
                next_events.insert(id.into_id());
            }
        })
    }
}