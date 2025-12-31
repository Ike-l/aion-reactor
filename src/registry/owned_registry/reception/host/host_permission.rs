use crate::registry::owned_registry::reception::host::access_map::access_map_permission::AccessMapPermission;

pub enum HostPermission {
    ReservationConflict,
    AccessMap(AccessMapPermission)
}