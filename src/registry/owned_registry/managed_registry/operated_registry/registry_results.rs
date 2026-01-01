pub enum RegistryOperatorAccessResult<AccessResult> {
    Found(AccessResult),
    ResourceNotFound,
}