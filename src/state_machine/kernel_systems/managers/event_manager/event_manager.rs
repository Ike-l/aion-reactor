use std::{pin::Pin, sync::Arc};

use tracing::{Level, event};

use crate::prelude::{CurrentEvents, EventMapper, KernelSystem, Memory, NextEvents, ProgramId, ProgramKey, Shared, SystemId, Unique};

pub struct EventManager;

impl KernelSystem for EventManager {
    fn system_id(&self) -> SystemId {
        SystemId::from("Event Manager")
    }

    fn init(&mut self, memory: &Memory, _kernel_program_id: &ProgramId, _kernel_program_key: &ProgramKey) {
        event!(Level::TRACE, status="Initialising", kernel_system_id = ?self.system_id());
        
        assert!(memory.insert(None, None, None, EventMapper::default()).unwrap().is_ok());
        assert!(memory.insert(None, None, None, NextEvents::default()).unwrap().is_ok());
        assert!(memory.insert(None, None, None, CurrentEvents::default()).unwrap().is_ok());
        
        event!(Level::TRACE, status="Initialised", kernel_system_id = ?self.system_id());
    }

    fn tick(&mut self, memory: &Arc<Memory>, _kernel_program_id: ProgramId, _kernel_program_key: ProgramKey) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let mut next_events = memory.resolve::<Unique<NextEvents>>(None, None, None, None).unwrap().unwrap();
            let mut current_events = memory.resolve::<Unique<CurrentEvents>>(None, None, None, None).unwrap().unwrap();
            let event_mapper = memory.resolve::<Shared<EventMapper>>(None, None, None, None).unwrap().unwrap();

            current_events.tick(&mut next_events);
            next_events.extend(event_mapper.tick(&current_events).into_iter().map(|e| e.clone()));
        })
    }
}