#[derive(Debug)]
pub enum AccessPermission {
    Access(bool),
    UnknownAccessId,
}

pub enum AccessRemoval {
    Split,
    UnknownAccessId
}