use crate::registry::owned_registry::reception::host::host_permission::{HostAccessPermission};

pub enum ReceptionAccessPermission {
    NoEntry,
    Host(HostAccessPermission)
}