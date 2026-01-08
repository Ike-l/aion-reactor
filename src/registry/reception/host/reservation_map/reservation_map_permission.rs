use crate::prelude::AccessRemovalResult;

pub enum ReservationMapPermission {
    ReservationConflict(bool)
}

pub enum ReservationMapUnReserveResult {
    AccessMap(AccessRemovalResult),
    NoReservation,
}