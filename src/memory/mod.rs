use crate::prelude::AccessMap;

pub mod access_map;

pub trait Memory {
    // 
    type Resource;
    type ResourceId;

    //
    type AccessMap: AccessMap;
    
    // 
    type Owner;

    // Results
    type Error;

    fn contains_resource(
        &self, 
        resource_id: &Self::ResourceId
    ) -> bool; 
    fn can_access_resource(
        &self, 
        access: &<Self::AccessMap as AccessMap>::Access, 
        owner: Option<&Self::Owner>
    ) -> bool;

    fn reserve(
        &self,
        owner: Self::Owner,
        accesses: Self::AccessMap,
    ) -> Option<Self::Error>;
    fn unreserve(
        &self,
        owner: &Self::Owner,
        accesses: Self::AccessMap,
    ) -> bool;

    fn insert<T: 'static>(
        &self,
        resource_id: &Self::ResourceId,
        resource: T,
    ) -> Option<Self::Error>;
    fn remove<T: 'static>(
        &self,
        resource_id: &Self::ResourceId
    ) -> Result<T, Self::Error>;

    fn get<T: 'static>(
        &self,
        resource_id: &Self::ResourceId
    ) -> Result<&T, Self::Error>;
    fn get_mut<T: 'static>(
        &self,
        resource_id: &Self::ResourceId
    ) -> Result<&mut T, Self::Error>;
    fn get_cloned<T: 'static + Clone>(
        &self,
        resource_id: &Self::ResourceId
    ) -> Result<T, Self::Error>;
}