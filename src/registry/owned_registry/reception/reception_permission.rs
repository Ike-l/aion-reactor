use crate::registry::owned_registry::reception::host::host_permission::HostPermission;

pub enum ReceptionPermission {
    NoEntry,
    Host(HostPermission)
}