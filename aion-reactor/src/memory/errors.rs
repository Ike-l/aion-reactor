use crate::memory::ResourceId;

#[derive(Debug, PartialEq)]
pub enum ResolveError {
    ConflictingAccess(ResourceId),
    ConflictingReservation(ResourceId),
    TooManyAccesses(ResourceId),
    InvalidProgramId,
    NoResource(ResourceId),
}

#[derive(Debug)]
pub enum DeResolveError {
    AccessDoesNotExist,
    AccessMismatch
}

#[derive(Debug)]
pub enum InsertError {
    ConcurrentAccess
}

#[derive(Debug, PartialEq)]
pub enum ReservationError {
    ConflictingReservation,
    ConcurrentAccess,
    ErrResource
}