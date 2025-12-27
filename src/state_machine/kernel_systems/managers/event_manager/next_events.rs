use std::{collections::HashSet, ops::Range};

use crate::prelude::EventId;

#[derive(Debug, Default)]
pub struct NextEvents(HashSet<EventId>);

impl NextEvents {
    pub fn insert<T: Into<EventId>>(&mut self, event: T) -> bool {
        self.0.insert(event.into())
    }

    pub fn remove(&mut self, event: &EventId) -> bool {
        self.0.remove(event)
    }

    pub fn extend(&mut self, events: impl Iterator<Item = EventId>) {
        self.0.extend(events);
    }

    pub fn drain(&mut self) -> impl Iterator<Item = EventId> {
        self.0.drain()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// no guarantees about the ordering
    pub fn get_range(&self, amount: Range<usize>) -> impl Iterator<Item = &EventId> {
        self.0.iter().take(amount.end)
    }
}