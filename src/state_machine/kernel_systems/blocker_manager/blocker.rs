use crate::id::Id;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Blocker(Id);

impl From<Id> for Blocker {
    fn from(value: Id) -> Self {
        Self(value)
    }
}

impl Blocker {
    pub fn id(&self) -> &Id {
        &self.0
    }
}
