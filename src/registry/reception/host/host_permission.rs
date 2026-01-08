use crate::prelude::{AccessPermission, ReservationMapUnReserveResult};

pub enum HostAccessPermission {
    ReservationConflict,
    AccessMap(AccessPermission)
}

pub enum HostReservationPermission {
    CurrentAccessConflict,
    ReservationConflict,
    Ok
}

pub enum HostUnReserveResult {
    ReservationMap(ReservationMapUnReserveResult)
}

pub enum HostDeAccessResult {
    Ok,
    UnknownAccessId
}