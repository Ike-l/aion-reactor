use crate::prelude::Id;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ProgramId(Id);

impl<T> From<T> for ProgramId 
where T: Into<Id>
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}