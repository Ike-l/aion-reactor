#[derive(Debug, PartialEq)]
pub enum OwnedRegistryAccessResult<AccessResult> {
    Found(AccessResult),
    NoEntry,
    AccessConflict,
    ReservationConflict,
    ResourceNotFound,
}

#[derive(Debug, PartialEq)]
pub enum OwnedRegistryReplaceResult<StoredResource> {
    NoEntry,
    Denied,
    Ok(Option<StoredResource>)
}