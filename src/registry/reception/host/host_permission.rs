use crate::prelude::{AccessPermission, ReservationMapUnReserve};

pub enum HostAccessPermission {
    ReservationConflict,
    AccessMap(AccessPermission)
}

pub enum HostReservationPermission {
    CurrentAccessConflict,
    ReservationConflict,
    Ok
}

pub enum HostUnReserve {
    ReservationMap(ReservationMapUnReserve)
}