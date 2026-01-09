pub enum ManagedRegistryAccessResult<AccessResult> {
    Found(AccessResult),
    ResourceNotFound,
    AccessFailure,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ManagedRegistryReplacementResult<AccessResult> {
    Found(AccessResult),
    NoOp,
    ResourceNotFound,
    AccessFailure
}