use crate::ids::Id;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct EventId(Id);

impl<T> From<T> for EventId 
where T: Into<Id>
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl EventId {
    pub fn get_id(&self) -> &Id {
        &self.0
    }
}
