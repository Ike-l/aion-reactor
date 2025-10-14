use std::{collections::HashMap, pin::Pin, sync::Arc};

use crate::{injection::injection_primitives::unique::Unique, memory::Memory, state_machine::kernel_systems::{event_manager::event::{CurrentEvents, Event, NextEvents}, KernelSystem}};

pub mod event;

pub struct EventManager {
    new_events: HashMap<Event, Event>
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            new_events: HashMap::new()
        }
    }
}

impl KernelSystem for EventManager {
    fn tick(&mut self, memory: &Arc<Memory>) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let mut next_events = memory.resolve::<Unique<NextEvents>>(None, None, None).unwrap().unwrap();
            let mut current_events = memory.resolve::<Unique<CurrentEvents>>(None, None, None).unwrap().unwrap();

            current_events.tick(&mut next_events);

            for event in current_events.read() {
                if let Some(new_event) = self.new_events.get(event) {
                    next_events.insert(new_event.clone());
                }
            }
        })
    }
}