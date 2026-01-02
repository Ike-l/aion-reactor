#[derive(Debug, PartialEq)]
pub enum RegistryAccessResult<AccessResult> {
    Found(AccessResult),
    NoEntry,
    AccessConflict,
    ReservationConflict,
    ResourceNotFound,
}

pub enum RegistryAccessPermission {
    Ok,
    NoEntry,
    ReservationConflict,
    AccessConflict
}

#[derive(Debug, PartialEq)]
pub enum RegistryReplacementResult<AccessResult> {
    NoEntry,
    ResourceNotFound,
    IncompatibleAccess,
    AccessConflict,
    ReservationConflict,
    Found(AccessResult)
}
