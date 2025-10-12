use std::{collections::HashSet, fmt::Debug};

use crate::state_machine::event::Event;

pub struct Criteria(Box<dyn Fn(&HashSet<Event>) -> bool + Send + Sync>);

impl Debug for Criteria {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "System Criteria")
    }
}

impl Criteria {
    pub fn test(&self, events: &HashSet<Event>) -> bool {
        self.0(events)
    }
}