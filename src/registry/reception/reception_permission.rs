use crate::prelude::{HostAccessPermission, HostReservationPermission, HostUnReserveResult};

pub enum ReceptionAccessPermission {
    NoEntry,
    Host(HostAccessPermission)
}

pub enum ReceptionReservationPermission {
    NoEntry,
    Host(HostReservationPermission)
}

pub enum ReceptionUnReserveResult {
    NoEntry,
    Host(HostUnReserveResult)
}

pub enum ReceptionDeAccessResult {
    Ok,
    UnknownAccessId
}