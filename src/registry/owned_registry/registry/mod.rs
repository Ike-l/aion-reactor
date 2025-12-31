pub mod managed_registry;

pub use managed_registry::{
    ManagedRegistry,
    RegistryManager,
};

use crate::registry::owned_registry::{reception::host::access_map::access::Access, registry::managed_registry::RegistryOperator, registry_result::RegistryResult};

pub trait Administrator {
    type RegistryManager: RegistryManager;
}

pub struct AdministratedRegistry<A: Administrator> {
    sync: parking_lot::RwLock<()>,
    registry: ManagedRegistry<A::RegistryManager>
}

impl<A: Administrator> AdministratedRegistry<A> {
    pub fn get<T: 'static, Ac: Access>(
        &self, 
        resource_id: &<<<A as Administrator>::RegistryManager as RegistryManager>::RegistryOperator as RegistryOperator>::ResourceId,
        access: &Ac
    ) -> RegistryResult<'_, T> {
        let _sync = self.sync.read();
        self.registry.get(resource_id, access)
    }
}

impl<A: Administrator> Default for AdministratedRegistry<A> {
    fn default() -> Self {
        Self {
            sync: parking_lot::RwLock::default(),
            registry: ManagedRegistry::default()
        }
    }
}