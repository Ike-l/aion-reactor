use crate::prelude::{HostAccessPermission, HostReservationPermission};

pub enum ReceptionAccessPermission {
    NoEntry,
    Host(HostAccessPermission)
}

pub enum ReceptionReservationPermission {
    NoEntry,
    Host(HostReservationPermission)
}