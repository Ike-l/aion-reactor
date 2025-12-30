use std::{collections::HashSet, ops::Range};

use crate::prelude::EventId;

#[derive(Debug, Default, Clone)]
pub struct CurrentEvents(HashSet<EventId>);

impl CurrentEvents {
    pub fn tick(&mut self, new_events: impl Iterator<Item = EventId>) {
        self.0.clear();

        self.0.extend(new_events);
    }

    pub fn read(&self) -> impl Iterator<Item = &EventId> {
        self.0.iter()
    }

    pub fn contains(&self, event: &EventId) -> bool {
        self.0.contains(event)
    }

    pub fn insert(&mut self, event: EventId) -> bool {
        self.0.insert(event)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// no guarantees about the ordering
    pub fn get_range(&self, amount: Range<usize>) -> impl Iterator<Item = &EventId> {
        self.0.iter().take(amount.end)
    }
}