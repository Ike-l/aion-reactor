use std::{pin::Pin, sync::Arc};

use tracing::{Level, event};

use crate::prelude::{CurrentEvents, DelayBuffer, DelayRegistry, KernelSystem, Memory, NextEvents, ProgramId, ProgramKey, ResourceId, Shared, SystemId, Unique};

pub struct DelayManager;

impl KernelSystem for DelayManager {
    fn system_id(&self) -> SystemId {
        SystemId::from("Delay Manager")
    }

    fn init(&mut self, memory: &Memory, _kernel_program_id: &ProgramId, _kernel_program_key: &ProgramKey) {
        event!(Level::DEBUG, "Inserting DelayRegistry");
        assert!(memory.insert(None, None, None, DelayRegistry::default()).unwrap().is_ok());
        
        event!(Level::DEBUG, "Inserting DelayBuffer");
        assert!(memory.insert(None, None, None, DelayBuffer::default()).unwrap().is_ok());   

        event!(Level::DEBUG, "Checking NextEvents");
        if !matches!(memory.contains_resource(None, &ResourceId::from_raw_heap::<NextEvents>(), None), Some(true)) {
            event!(Level::WARN, "NextEvents Not Found");   
        }

        event!(Level::DEBUG, "Checking CurrentEvents");
        if !matches!(memory.contains_resource(None, &ResourceId::from_raw_heap::<CurrentEvents>(), None), Some(true)) {
            event!(Level::WARN, "CurrentEvents Not Found");   
        }
    }

    fn tick(&mut self, memory: &Arc<Memory>, _kernel_program_id: ProgramId, _kernel_program_key: ProgramKey) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let mut buffer = memory.resolve::<Unique<DelayBuffer>>(None, None, None, None).unwrap().unwrap();
            let registry = memory.resolve::<Shared<DelayRegistry>>(None, None, None, None).unwrap().unwrap();
            let mut next_events = memory.resolve::<Unique<NextEvents>>(None, None, None, None).unwrap().unwrap();
            let current_events= memory.resolve::<Shared<CurrentEvents>>(None, None, None, None).unwrap().unwrap();
            
            event!(Level::DEBUG, old_next_event_count = next_events.len());

            buffer.tick(&registry, &current_events, &mut next_events);
            event!(Level::DEBUG, new_next_event_count = next_events.len());
            event!(Level::TRACE, next_events = ?next_events);
        })
    }
}