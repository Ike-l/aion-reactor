#[derive(Debug, PartialEq)]
pub enum OwnedRegistryAccessResult<AccessResult> {
    Found(AccessResult),
    NoEntry,
    AccessConflict,
    ReservationConflict,
    ResourceNotFound,
}

pub enum OwnedRegistryReplaceResult<Resource> {
    NoEntry,
    Denied,
    Ok(Option<Resource>)
}