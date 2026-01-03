use tracing::{Instrument, Level, span};

use crate::prelude::{AccessKey, AccessPermission, Accessor, HostAccessPermission, Key, ManagedRegistry, ManagedRegistryAccessResult, Reception, ReceptionAccessPermission, RegistryAccessPermission, RegistryAccessResult, RegistryReplacementResult, ReserverKey, ResourceKey};

pub mod managed_registry;
pub mod reception;
pub mod registry_results;

pub struct Registry<
    AccessId, 
    ReserverId,
    Access,
    ResourceId,
    KeyId,
    StoredResource,
> {
    // can make tokio and everything async?
    sync: parking_lot::Mutex<()>,
    registry: ManagedRegistry<ResourceId, StoredResource>,
    reception: Reception<AccessId, ReserverId, Access, ResourceId, KeyId>
}

impl<
    ReserverId: ReserverKey,
    Access: Accessor<StoredResource = StoredResource>,
    ResourceId: ResourceKey + AccessKey + Clone,
    KeyId: Key,
    StoredResource,
> Registry<ResourceId, ReserverId, Access, ResourceId, KeyId, StoredResource> {
    fn permits_access(
        &self,
        resource_id: &ResourceId,
        access: &Access,
        reserver_id: Option<&ReserverId>,
        key: Option<&KeyId>,
    ) -> RegistryAccessPermission {
        let span = span!(Level::DEBUG, "Registry Permits Access");
        let _enter = span.enter();

        match self.reception.permits_access(resource_id, access, reserver_id, key) {
            ReceptionAccessPermission::NoEntry => RegistryAccessPermission::NoEntry,
            ReceptionAccessPermission::Host(HostAccessPermission::ReservationConflict) => RegistryAccessPermission::ReservationConflict,
            ReceptionAccessPermission::Host(HostAccessPermission::AccessMap(AccessPermission::Access(false))) => RegistryAccessPermission::AccessConflict,
            ReceptionAccessPermission::Host(HostAccessPermission::AccessMap(AccessPermission::Access(true))) | 
            ReceptionAccessPermission::Host(HostAccessPermission::AccessMap(AccessPermission::UnknownAccessId)) => RegistryAccessPermission::Ok
        }
    }

    pub fn access(
        &self, 
        resource_id: ResourceId,
        access: Access,
        reserver_id: Option<&ReserverId>,
        key: Option<&KeyId>,
    ) -> RegistryAccessResult<Access::AccessResult<'_, Access::Resource>> { 
        let span = span!(Level::DEBUG, "Registry Access");
        let _enter = span.enter();

        let _sync = self.sync.lock();
        match self.permits_access(&resource_id, &access, reserver_id, key) {
            RegistryAccessPermission::NoEntry => RegistryAccessResult::NoEntry,
            RegistryAccessPermission::ReservationConflict => RegistryAccessResult::ReservationConflict,
            RegistryAccessPermission::AccessConflict => RegistryAccessResult::AccessConflict,
            RegistryAccessPermission::Ok => {
                unsafe { 
                    match self.registry.access(&resource_id, &access) {
                        ManagedRegistryAccessResult::ResourceNotFound => RegistryAccessResult::ResourceNotFound,
                        ManagedRegistryAccessResult::Found(result) => {
                            self.reception.record_access(resource_id, access, reserver_id);
                            RegistryAccessResult::Found(result)
                        }
                    }
                }
            },
        }
    }

    pub fn accessed_replacement(
        &self,
        resource_id: ResourceId,
        access: Access,
        reserver_id: Option<&ReserverId>,
        key: Option<&KeyId>,
        resource: Option<StoredResource>,
    ) -> RegistryReplacementResult<Access::AccessResult<'_, Access::StoredResource>> {
        let span = span!(Level::DEBUG, "Registry Accessed Replacement");
        let _enter = span.enter();

        let _sync = self.sync.lock();
        match self.permits_access(&resource_id, &access, reserver_id, key) {
            RegistryAccessPermission::NoEntry => RegistryReplacementResult::NoEntry,
            RegistryAccessPermission::AccessConflict => RegistryReplacementResult::AccessConflict,
            RegistryAccessPermission::ReservationConflict => RegistryReplacementResult::ReservationConflict,
            RegistryAccessPermission::Ok => {
                match unsafe { self.registry.accessed_replacement(resource_id.clone(), resource, &access) } {
                    ManagedRegistryAccessResult::ResourceNotFound => RegistryReplacementResult::ResourceNotFound,
                    ManagedRegistryAccessResult::Found(access_result) => {
                        self.reception.record_access(resource_id, access, reserver_id);
                        RegistryReplacementResult::Found(access_result)
                    }
                }
            }
        }
    }

    // remove has to check *all* accesses, not just conflicts
    // remove has to check reservations the same

    // can reservations be made on a resource that doesnt exist? No!
}

impl<
    AccessId,
    ReserverId,
    Access,
    ResourceId,
    Key,
    StoredResource,
> Default for Registry<AccessId, ReserverId, Access, ResourceId, Key, StoredResource> {
    fn default() -> Self {
        Self {
            sync: parking_lot::Mutex::default(),
            reception: Reception::default(),
            registry: ManagedRegistry::default()
        }
    }
}