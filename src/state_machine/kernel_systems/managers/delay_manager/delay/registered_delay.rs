use crate::prelude::EventId;

pub struct RegisteredDelay {
    pub activated_by: EventId,
    pub then_inserts: EventId,
    pub delayed_by: Option<EventId>
}