use std::{pin::Pin, sync::Arc};

use tracing::{Level, event, span};

use crate::prelude::{ExecutionGraph, KernelSystem, Memory, NextBlockers, NextEvents, Processor, ProcessorSystemRegistry, ProgramId, ProgramKey, ResourceId, Shared, StateMachine, StoredSystem, SystemEventRegistry, SystemId, SystemMetadata, Unique};

#[derive(Debug)]
pub struct BlockingProcessor;

impl BlockingProcessor {
    pub fn insert_system(state_machine: &StateMachine, system_id: SystemId, system_metadata: SystemMetadata, stored_system: StoredSystem) -> Option<Option<SystemMetadata>> {
        let mut system_registry = state_machine.memory.resolve::<Unique<ProcessorSystemRegistry>>(None, None, None, None)?.ok()?;
        Processor::insert_system(state_machine, &mut system_registry.0, system_id, system_metadata, stored_system)
    }
}

impl KernelSystem for BlockingProcessor {
    fn system_id(&self) -> SystemId {
        SystemId::from("Blocking Processor")
    }

    fn init(&mut self, memory: &Memory, kernel_program_id: &ProgramId, kernel_program_key: &ProgramKey) {
        event!(Level::DEBUG, "Inserting ProcessorSystemRegistry");
        assert!(memory.insert(None, None, None, ProcessorSystemRegistry::default()).unwrap().is_ok());

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

        event!(Level::DEBUG, "Checking Arc<tokio::runtime::Runtime>");
        if !matches!(memory.contains_resource(Some(kernel_program_id), &ResourceId::from_raw_heap::<Arc<tokio::runtime::Runtime>>(), Some(kernel_program_key)), Some(true)) {
            event!(Level::WARN, "Arc<tokio::runtime::Runtime> Not Found")
        }
        
        event!(Level::DEBUG, "Checking threadpool::ThreadPool");
        if !matches!(memory.contains_resource(Some(kernel_program_id), &ResourceId::from_raw_heap::<threadpool::ThreadPool>(), Some(kernel_program_key)), Some(true)) {
            event!(Level::WARN, "threadpool::ThreadPool Not Found")
        }
    }

    fn tick(&mut self, memory: &Arc<Memory>, kernel_program_id: ProgramId, kernel_program_key: ProgramKey) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let system_registry = memory.resolve::<Shared<ProcessorSystemRegistry>>(None, None, None, None).unwrap().unwrap();
            
            let systems = Processor::get_systems(&memory, &system_registry.0);
            
            event!(Level::DEBUG, executing_systems_count=systems.len(), "Executing Systems");

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
    
            let independent_systems = Processor::divide_independent_by_aliasing(systems);
    
            let execution_graphs = independent_systems
                .map(|systems| {
                    systems.into_iter().map(|(id, system_metadata)| {
                        (id, system_metadata.ordering())
                    }).collect::<Vec<_>>()
                })
                .map(|systems| {
                    std::sync::RwLock::new(ExecutionGraph::new(&systems))
                })
                .collect::<Vec<_>>();

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

            event!(Level::DEBUG, "Executing");
            let results = Processor::execute(
                &memory,
                Arc::new(execution_graphs),
                &system_registry.0,
                &threadpool,
                &runtime
            ).await;

            let mut next_events = memory.resolve::<Unique<NextEvents>>(None, None, None, None).unwrap().unwrap();
            let mut next_blockers = memory.resolve::<Unique<NextBlockers>>(None, None, None, None).unwrap().unwrap();

            let system_event_registry = memory.resolve::<Shared<SystemEventRegistry>>(None, None, None, None).unwrap().unwrap();

            let results_span = span!(Level::DEBUG, "Results");
            let _enter = results_span.enter();
            
            for (system_id, result) in results {
                event!(Level::TRACE, system_id=?system_id, result=?result, "System Returned Result");
                result.act(
                    &system_id,
                    &mut next_events,
                    &mut next_blockers,
                    &system_event_registry,
                    results_span.clone()
                );
            }
        })        
    }
}