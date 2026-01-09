#[derive(Debug, PartialEq, Eq)]
pub enum OperatedRegistryAccessResult<AccessResult> {
    Found(AccessResult),
    ResourceNotFound,
}

#[derive(Debug, PartialEq, Eq)]
pub enum OperatedRegistryReplacementResult<AccessResult> {
    Found(AccessResult),
    NoOp,
    ResourceNotFound,
    AccessFailure
}