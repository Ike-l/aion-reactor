use std::collections::HashSet;

use crate::state_machine::kernel_systems::event_manager::event::Event;

#[derive(Debug, Default)]
pub struct NextEvents(HashSet<Event>);

impl NextEvents {
    pub fn insert<T: Into<Event>>(&mut self, event: T) -> bool {
        self.0.insert(event.into())
    }

    pub fn remove(&mut self, event: &Event) -> bool {
        self.0.remove(event)
    }

    pub fn extend(&mut self, events: impl Iterator<Item = Event>) {
        self.0.extend(events);
    }

    pub fn drain(&mut self) -> impl Iterator<Item = Event> {
        self.0.drain()
    }
}