use crate::registry::owned_registry::reception::host::access_map::access_map_permission::AccessPermission;

pub enum HostAccessPermission {
    ReservationConflict,
    AccessMap(AccessPermission)
}