use std::collections::HashSet;

use crate::state_machine::kernel_systems::event_manager::{event::Event, next_events::NextEvents};

#[derive(Debug, Default, Clone)]
pub struct CurrentEvents(HashSet<Event>);

impl CurrentEvents {
    pub fn tick(&mut self, new_events: &mut NextEvents) {
        self.0.clear();

        self.0.extend(new_events.drain());
    }

    pub fn read(&self) -> impl Iterator<Item = &Event> {
        self.0.iter()
    }

    pub fn contains(&self, event: &Event) -> bool {
        self.0.contains(event)
    }

    pub fn insert(&mut self, event: Event) -> bool {
        self.0.insert(event)
    }
}