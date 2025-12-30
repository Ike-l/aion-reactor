use crate::prelude::EventId;

pub struct RegisteredDelay {
    when: EventId,
    ends_with: EventId,
    delayed_by: Option<EventId>
}