#[derive(Debug, PartialEq)]
pub enum AccessResult<'a, T> {
    Shared(&'a T),
    Unique(&'a mut T),
    Owned(T),
    Fail
}
