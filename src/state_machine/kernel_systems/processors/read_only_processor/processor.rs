use std::{pin::Pin, sync::Arc};

use tracing::{Level, event};

use crate::prelude::{KernelSystem, Memory, NextEvents, Processor, ProgramId, ProgramKey, ReadOnlySystemRegistry, ResourceId, Shared, StateMachine, StoredSystem, SystemEventRegistry, SystemId, SystemMetadata, SystemResult, Unique};

pub struct ReadOnlyProcessor;

impl ReadOnlyProcessor {
    pub fn insert_system(
        state_machine: &StateMachine, 
        system_id: SystemId, 
        system_metadata: SystemMetadata, 
        stored_system: StoredSystem
    ) -> Option<Option<SystemMetadata>> {
        let mut system_registry = state_machine.memory.resolve::<Unique<ReadOnlySystemRegistry>>(None, None, None, None)?.ok()?;
        Processor::insert_system(state_machine, system_registry.ref_mut_generic(), system_id, system_metadata, stored_system)
    }
}

impl KernelSystem for ReadOnlyProcessor {
    fn system_id(&self) -> SystemId {
        SystemId::from("ReadOnly Processor")    
    }

    fn init(&mut self, memory: &Memory, kernel_program_id: &ProgramId, kernel_program_key: &ProgramKey) {
        event!(Level::TRACE, status="Initialising", kernel_system_id = ?self.system_id());
        assert!(matches!(memory.contains_resource(Some(kernel_program_id), &ResourceId::from_raw_heap::<Arc<tokio::runtime::Runtime>>(), Some(kernel_program_key)), Some(true)));
        assert!(matches!(memory.contains_resource(Some(kernel_program_id), &ResourceId::from_raw_heap::<threadpool::ThreadPool>(), Some(kernel_program_key)), Some(true)));
        
        assert!(memory.insert(None, None, None, ReadOnlySystemRegistry::default()).unwrap().is_ok());
        assert!(memory.insert(None, None, None, SystemEventRegistry::default()).unwrap().is_ok());
        
        event!(Level::TRACE, status="Initialised", kernel_system_id = ?self.system_id());
    }
    
    fn tick(&mut self, memory: &Arc<Memory>, kernel_program_id: ProgramId, kernel_program_key: ProgramKey) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let system_registry = memory.resolve::<Shared<ReadOnlySystemRegistry>>(None, None, None, None).unwrap().unwrap();
            let systems = Processor::get_systems(self.system_id(), &memory, system_registry.ref_generic());
            
            let systems = systems.into_iter().map(|(id, _)| id.clone()).collect::<Vec<_>>();
            
            /*
            let system_map = Arc::new(
            memory.resolve::<Shared<ReadOnlySystemRegistry>>(None, None, None, None)
                .unwrap().unwrap().0
                .into_map()
                .collect::<HashMap<_, _>>()
        ); */

            let runtime = memory.resolve::<Shared<Arc<tokio::runtime::Runtime>>>(
                Some(&kernel_program_id), 
                None, 
                None, 
                Some(&kernel_program_key)
            ).unwrap().unwrap();

            let threadpool = memory.resolve::<Shared<threadpool::ThreadPool>>(
                Some(&kernel_program_id), 
                None, 
                None, 
                Some(&kernel_program_key)
            ).unwrap().unwrap();

            let results = Processor::execute_fast(
                self.system_id(),
                &memory, 
                systems, 
                system_registry.ref_generic(), 
                &threadpool, 
                &runtime
            ).await;

            let system_event_registry = memory.resolve::<Shared<SystemEventRegistry>>(None, None, None, None).unwrap().unwrap();
            let mut next_events = memory.resolve::<Unique<NextEvents>>(None, None, None, None).unwrap().unwrap();

            for (id, result) in results {
                if matches!(result, SystemResult::Conditional(true)) {
                    if let Some(events) = system_event_registry.get(&id) {
                        next_events.extend(events.clone().into_iter());
                    } else {
                        println!("Warn: `ReadOnlySystem` returned `SystemResult::Conditional(true)` without an `Event` mapping\nSuggestion: insert `SystemEventRegistry` with system `Id` (`Source`), with the list of `Event`s you want to spawn in `NextEvents`");
                    }
                }
            }
        })
    }
}