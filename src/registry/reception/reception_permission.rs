use crate::prelude::{HostAccessPermission, HostReservationPermission, HostUnReserve};

pub enum ReceptionAccessPermission {
    NoEntry,
    Host(HostAccessPermission)
}

pub enum ReceptionReservationPermission {
    NoEntry,
    Host(HostReservationPermission)
}

pub enum ReceptionUnReserve {
    NoEntry,
    Host(HostUnReserve)
}