use crate::prelude::AccessPermission;

pub enum HostAccessPermission {
    ReservationConflict,
    AccessMap(AccessPermission)
}

pub enum HostReservationPermission {
    CurrentAccessConflict,
    ReservationConflict,
    Ok
}