use crate::prelude::AccessRemoval;

pub enum ReservationMapPermission {
    ReservationConflict(bool)
}

pub enum ReservationMapUnReserve {
    AccessMap(AccessRemoval),
    NoReservation,
}