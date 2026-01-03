#[derive(Debug, PartialEq)]
pub enum RegistryAccessResult<AccessResult> {
    Found(AccessResult),
    NoEntry,
    AccessConflict,
    ReservationConflict,
    ResourceNotFound,
    AccessFailure,
}

pub enum RegistryAccessPermission {
    Ok,
    NoEntry,
    ReservationConflict,
    AccessConflict
}

#[derive(Debug, PartialEq)]
pub enum RegistryReplacementResult<AccessResult> {
    Found(AccessResult),
    NoEntry,
    ResourceNotFound,
    IncompatibleAccess,
    AccessConflict,
    ReservationConflict,
    AccessFailure,
}
