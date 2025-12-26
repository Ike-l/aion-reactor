use std::{collections::HashSet, fmt::Debug};

use crate::prelude::EventId;


pub struct Criteria(Box<dyn Fn(&HashSet<&EventId>) -> bool + Send + Sync>);

impl Debug for Criteria {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "System Criteria")
    }
}

impl Criteria {
    pub fn new(criteria: impl Fn(&HashSet<&EventId>) -> bool + Send + Sync + 'static) -> Self {
        Self(
            Box::new(criteria)
        )
    }

    pub fn test(&self, events: &HashSet<&EventId>) -> bool {
        self.0(events)
    }
}