use crate::prelude::Id;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct SystemId(Id);

impl<T> From<T> for SystemId 
where T: Into<Id>
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl SystemId {
    pub fn into_id(self) -> Id {
        self.0
    }
}
