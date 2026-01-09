use crate::prelude::ReceptionReservationPermission;

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
    NoOp,
    AccessFailure,
}

pub enum RegistryReservationResult {
    Reception(ReceptionReservationPermission),
    NoResource,
}

pub enum RegistryUnReserveResult {
    NoEntry,
    NoReservation,
    UnknownResourceId,
    Ok
}

pub enum RegistryDeAccessResult {
    Ok,
    NoEntry,
    UnknownResourceId
}