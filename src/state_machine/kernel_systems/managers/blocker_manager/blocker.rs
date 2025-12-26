use crate::prelude::SystemId;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Blocker(SystemId);

impl<T> From<T> for Blocker 
where T: Into<SystemId>
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl Blocker {
    pub fn get_blocks(&self) -> &SystemId {
        &self.0
    }

    pub fn into(self) -> SystemId {
        self.0
    }
}
