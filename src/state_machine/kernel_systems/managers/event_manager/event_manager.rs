use std::{pin::Pin, sync::Arc};

use tracing::{Level, event};

use crate::prelude::{CurrentEvents, EventMapper, KernelSystem, Memory, NextEvents, ProgramId, ProgramKey, SystemEventRegistry, SystemId, Unique};

pub struct EventManager;

impl KernelSystem for EventManager {
    fn system_id(&self) -> SystemId {
        SystemId::from("Event Manager")
    }

    fn init(&mut self, memory: &Memory, _kernel_program_id: &ProgramId, _kernel_program_key: &ProgramKey) {
        event!(Level::DEBUG, "Inserting NextEvents");
        assert!(memory.insert(None, None, None, NextEvents::default()).unwrap().is_ok());

        event!(Level::DEBUG, "Inserting CurrentEvents");
        assert!(memory.insert(None, None, None, CurrentEvents::default()).unwrap().is_ok());

        event!(Level::DEBUG, "Inserting EventMapper");
        assert!(memory.insert(None, None, None, EventMapper::default()).unwrap().is_ok());

        event!(Level::DEBUG, "Inserting SystemEventRegistry");
        assert!(memory.insert(None, None, None, SystemEventRegistry::default()).unwrap().is_ok());
    }

    fn tick(&mut self, memory: &Arc<Memory>, _kernel_program_id: ProgramId, _kernel_program_key: ProgramKey) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let mut next_events = memory.resolve::<Unique<NextEvents>>(None, None, None, None).unwrap().unwrap();
            let mut current_events = memory.resolve::<Unique<CurrentEvents>>(None, None, None, None).unwrap().unwrap();

            event!(Level::DEBUG, old_current_event_count = current_events.len());

            current_events.tick(next_events.drain());
            event!(Level::DEBUG, new_current_event_count = current_events.len());
            event!(Level::TRACE, current_events = ?current_events);
        })
    }
}