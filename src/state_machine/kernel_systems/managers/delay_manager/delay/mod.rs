use crate::prelude::EventId;

pub mod registered_delay;

pub struct Delay {
    pub then_inserts: EventId,
    pub delayed_by: Option<EventId>
}

impl Delay {
    pub fn new(then_inserts: EventId, delayed_by: Option<EventId>) -> Self {
        Self { then_inserts, delayed_by }
    }
}
