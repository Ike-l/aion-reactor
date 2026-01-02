pub trait Accessor {
    type StoredResource;
    type Resource: 'static;
    type AccessResult<'a, T> where T: 'a;

    fn can_access(&self, other: &Self) -> bool;
    fn can_replace_resource(&self) -> bool;

    fn merge_access(&mut self, other: Self);
    fn split_access(&mut self, other: &Self);

    fn access<'a>(&self, resource: &'a Self::StoredResource) -> Self::AccessResult<'a, Self::Resource>;
}