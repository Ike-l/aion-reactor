use std::fmt::Debug;

pub trait Accessor: Debug {
    type StoredResource;
    type Resource: 'static;
    type AccessResult<'a, T> where T: 'a;

    fn can_access(&self, other: &Self) -> bool;

    // in future can provide a resource id to make it more dynamic
    fn can_insert(&self) -> bool;
    fn can_remove(&self) -> bool;

    fn merge_access(&mut self, other: Self);
    fn split_access(&mut self, other: &Self);

    /// Called when `resource` is being accessed with `self`
    fn access<'a>(&self, resource: &'a Self::StoredResource) -> Self::AccessResult<'a, Self::Resource>;

    /// Called when `resource` is being removed
    fn remove<'a>(&self, resource: Self::StoredResource) -> Self::AccessResult<'a, Self::StoredResource>;
    
    /// Called when `resource` is being inserted
    fn insert<'a>(&self, _resource: &'a Self::StoredResource) {}
}