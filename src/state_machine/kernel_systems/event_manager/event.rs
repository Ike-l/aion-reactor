use crate::{id::Id};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Event(Id);

impl Event {
    pub fn new(name: String) -> Self {
        Self::from(Id(name))
    }
}

impl From<Id> for Event {
    fn from(value: Id) -> Self {
        Self(value)
    }
}

impl Event {
    pub fn id(&self) -> &Id {
        &self.0
    }
}
