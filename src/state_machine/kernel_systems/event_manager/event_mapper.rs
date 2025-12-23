use std::collections::{HashMap, HashSet};

use crate::state_machine::kernel_systems::event_manager::{event::Event, prelude::CurrentEvents};

#[derive(Default)]
pub struct EventMapper(HashMap<Event, HashSet<Event>>);

impl EventMapper {
    pub fn insert(&mut self, from: Event, to: Event) -> bool {
        self.0.entry(from).or_default().insert(to)
    }

    pub fn tick(&self, current_events: &CurrentEvents) -> HashSet<&Event> {
        current_events.read().fold(HashSet::new(), |mut acc, event| {
            if let Some(new_events) = self.0.get(event) {
                acc.extend(new_events);
            }

            acc
        })
    }
}