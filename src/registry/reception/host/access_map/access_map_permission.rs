#[derive(Debug)]
pub enum AccessPermission {
    Access(bool),
    UnknownAccessId,
}

pub enum AccessRemovalResult {
    Split,
    UnknownAccessId
}