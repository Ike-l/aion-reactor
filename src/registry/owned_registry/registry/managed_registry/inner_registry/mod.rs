use std::{collections::HashMap, hash::Hash};

use crate::registry::owned_registry::{reception::host::{Accessor, access_map::access::Access}, registry::managed_registry::inner_registry::resource::Resource, registry_result::{GetResult, RegistryResult}};

pub mod resource;

pub trait RegistryOperator {
    type ResourceId: Eq + Hash;
    type Resource: Resource;    
}

pub struct OperatedRegistry<R: RegistryOperator> {
    registry: HashMap<R::ResourceId, R::Resource>
}

impl<R: RegistryOperator> OperatedRegistry<R> {
    pub fn get<T: 'static, A: Access>(
        &self, 
        resource_id: &R::ResourceId,
        access: &A
    ) -> RegistryResult<'_, T> {
        if let Some(resource) = self.registry.get(resource_id) {
            if let Some(resource) = resource.get(access) {
                RegistryResult::Success(resource)
            } else {
                RegistryResult::IncompatibleStoredResource
            }
        } else {
            RegistryResult::NoResource
        }
    }
}

impl<R: RegistryOperator> Default for OperatedRegistry<R> {
    fn default() -> Self {
        Self {
            registry: HashMap::new()
        }
    }
}