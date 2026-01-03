use crate::prelude::HostAccessPermission;

pub enum ReceptionAccessPermission {
    NoEntry,
    Host(HostAccessPermission)
}
