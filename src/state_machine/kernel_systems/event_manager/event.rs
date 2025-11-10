use std::collections::HashSet;

use crate::{id::Id, state_machine::transition_phases::TransitionPhase};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Event(Id);

impl From<Id> for Event {
    fn from(value: Id) -> Self {
        Self(value)
    }
}

impl From<TransitionPhase> for Event {
    fn from(value: TransitionPhase) -> Self {
        Self::from(Id::from(value))
    }
}

impl Event {
    pub fn id(&self) -> &Id {
        &self.0
    }
}

#[derive(Debug, Default)]
pub struct NextEvents(HashSet<Event>);

impl NextEvents {
    pub fn insert<T: Into<Event>>(&mut self, event: T) -> bool {
        self.0.insert(event.into())
    }

    pub fn remove(&mut self, event: &Event) -> bool {
        self.0.remove(event)
    }
}

#[derive(Debug, Default, Clone)]
pub struct CurrentEvents(HashSet<Event>);

impl CurrentEvents {
    pub fn tick(&mut self, new_events: &mut NextEvents) {
        self.0.clear();

        self.0.extend(new_events.0.drain());
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