pub enum ManagedRegistryAccessResult<AccessResult> {
    Found(AccessResult),
    ResourceNotFound,
    AccessFailure,
}