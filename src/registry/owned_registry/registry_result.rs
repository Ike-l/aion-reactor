pub enum RegistryResult<'a, T> {
    Success(GetResult<'a, T>),
    NoEntry,
    AccessMapConflict,
    ReservationConflict,
    NoResource,
    IncompatibleStoredResource,
}

pub enum GetResult<'a, T> {
    Shared(&'a T),
    Unique(&'a mut T),
}