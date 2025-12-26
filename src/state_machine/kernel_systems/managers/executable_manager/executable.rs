use crate::prelude::EventId;

#[derive(Clone)]
pub struct Executable {
    pub label: String,
    pub trigger: EventId,
}

impl Executable {
    pub fn new(label: String, trigger: EventId) -> Self {
        Self {
            label,
            trigger
        }
    }
}
