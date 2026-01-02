#[derive(Debug, PartialEq)]
pub enum AccessResult<'a, T> {
    Shared(&'a T),
    Unique(&'a T),
    Owned(T),
    Fail
}
