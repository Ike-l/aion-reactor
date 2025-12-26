use std::collections::{HashMap, HashSet};

use crate::prelude::{CurrentEvents, EventId};

#[derive(Default)]
pub struct EventMapper(HashMap<EventId, HashSet<EventId>>);

impl EventMapper {
    pub fn insert(&mut self, from: EventId, to: EventId) -> bool {
        self.0.entry(from).or_default().insert(to)
    }

    pub fn tick(&self, current_events: &CurrentEvents) -> HashSet<&EventId> {
        current_events.read().fold(HashSet::new(), |mut acc, event| {
            if let Some(new_events) = self.0.get(event) {
                acc.extend(new_events);
            }

            acc
        })
    }
}