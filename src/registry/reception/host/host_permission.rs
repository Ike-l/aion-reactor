use crate::prelude::AccessPermission;

pub enum HostAccessPermission {
    ReservationConflict,
    AccessMap(AccessPermission)
}