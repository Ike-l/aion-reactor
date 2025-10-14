use crate::memory::ResourceId;

#[derive(Debug)]
pub enum ResolveError {
    ConflictingAccess(ResourceId),
    ConflictingReservation(ResourceId),
    InvalidProgramId,
    NoResource(ResourceId),
}

#[derive(Debug)]
pub enum DeResolveError {
    AccessDoesNotExist,
    AccessMismatch
}