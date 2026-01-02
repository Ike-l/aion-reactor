pub enum OperatedRegistryAccessResult<AccessResult> {
    Found(AccessResult),
    ResourceNotFound,
}