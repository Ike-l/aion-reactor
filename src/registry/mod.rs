use crate::prelude::{AccessKey, AccessPermission, Accessor, HostAccessPermission, Key, ManagedRegistry, ManagedRegistryAccessResult, OwnedRegistryAccessResult, OwnedRegistryReplaceResult, Reception, ReceptionAccessPermission, ReserverKey, ResourceKey};

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
    pub fn access(
        &self, 
        resource_id: ResourceId,
        access: Access,
        reserver_id: Option<&ReserverId>,
        key: Option<&KeyId>,
    ) -> OwnedRegistryAccessResult<Access::AccessResult<'_, Access::Resource>> { 
        let _sync = self.sync.lock();
        match self.reception.permits_access(&resource_id, Some(&access), reserver_id, key) {
            ReceptionAccessPermission::NoEntry => OwnedRegistryAccessResult::NoEntry,
            ReceptionAccessPermission::Host(HostAccessPermission::ReservationConflict) => OwnedRegistryAccessResult::ReservationConflict,
            ReceptionAccessPermission::Host(HostAccessPermission::AccessMap(AccessPermission::Insert(_))) => unreachable!("Access is Some"),
            ReceptionAccessPermission::Host(HostAccessPermission::AccessMap(AccessPermission::Access(false))) => OwnedRegistryAccessResult::AccessConflict,
            ReceptionAccessPermission::Host(HostAccessPermission::AccessMap(AccessPermission::Access(true))) | 
            ReceptionAccessPermission::Host(HostAccessPermission::AccessMap(AccessPermission::UnknownAccessId)) => {
                match unsafe { self.registry.access(&resource_id, &access) } {
                    ManagedRegistryAccessResult::ResourceNotFound => OwnedRegistryAccessResult::ResourceNotFound,
                    ManagedRegistryAccessResult::Found(result) => {
                        self.reception.record_access(resource_id, access, reserver_id);
                        OwnedRegistryAccessResult::Found(result)
                    }
                }
            }
        }
    }

    pub fn replace(
        &self,
        resource_id: ResourceId,
        key: Option<&KeyId>,
        resource: StoredResource,
    ) -> OwnedRegistryReplaceResult<StoredResource> {
        let _sync = self.sync.lock();
        match self.reception.permits_access(&resource_id, None, None, key) {
            ReceptionAccessPermission::NoEntry => OwnedRegistryReplaceResult::NoEntry,

            ReceptionAccessPermission::Host(HostAccessPermission::AccessMap(AccessPermission::Access(_))) | 
            ReceptionAccessPermission::Host(HostAccessPermission::ReservationConflict) => unreachable!("Access is None"),

            ReceptionAccessPermission::Host(HostAccessPermission::AccessMap(AccessPermission::Insert(false))) => OwnedRegistryReplaceResult::Denied,

            ReceptionAccessPermission::Host(HostAccessPermission::AccessMap(AccessPermission::Insert(true))) |
            ReceptionAccessPermission::Host(HostAccessPermission::AccessMap(AccessPermission::UnknownAccessId)) => {
                OwnedRegistryReplaceResult::Ok(unsafe { self.registry.replace(resource_id, resource) })
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