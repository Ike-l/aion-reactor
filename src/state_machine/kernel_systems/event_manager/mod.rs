use std::{collections::HashMap, pin::Pin, sync::Arc};

use crate::{injection::injection_primitives::{shared::Shared, unique::Unique}, memory::Memory, state_machine::kernel_systems::{event_manager::event::{CurrentEvents, Event, NextEvents}, KernelSystem}};

pub mod event;

pub struct EventManager;

pub struct EventMapper(HashMap<Event, Event>);

impl EventManager {
    pub fn new(memory: &Arc<Memory>) -> Self {
        memory.insert(None, None, EventMapper(HashMap::new()));
        Self
    }
}

impl KernelSystem for EventManager {
    fn tick(&mut self, memory: &Arc<Memory>) -> Pin<Box<dyn Future<Output = ()> + '_ + Send>> {
        let memory = Arc::clone(&memory);
        Box::pin(async move {
            let mut next_events = memory.resolve::<Unique<NextEvents>>(None, None, None).unwrap().unwrap();
            let mut current_events = memory.resolve::<Unique<CurrentEvents>>(None, None, None).unwrap().unwrap();

            current_events.tick(&mut next_events);

            let new_events = memory.resolve::<Shared<EventMapper>>(None, None, None).unwrap().unwrap();

            for event in current_events.read() {
                if let Some(new_event) = new_events.0.get(event) {
                    next_events.insert(new_event.clone());
                }
            }
        })
    }
}