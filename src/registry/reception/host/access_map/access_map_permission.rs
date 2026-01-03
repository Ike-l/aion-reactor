#[derive(Debug)]
pub enum AccessPermission {
    Access(bool),
    UnknownAccessId,
}