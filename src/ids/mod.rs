pub mod system_id;
pub mod program_id;
pub mod event_id;
pub mod blocker_id;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Id(String);

impl<T> From<T> for Id 
where T: Into<String>
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}