pub mod managed_registry;
pub mod reception;
pub mod registry_results;

pub use reception::{
    Reception, 
};

use crate::registry::owned_registry::{managed_registry::{ManagedRegistry, operated_registry::ResourceKey, registry_results::ManagedRegistryAccessResult}, reception::{gate::Key, host::{Accessor, access_map::{AccessKey, access_map_permission::AccessPermission}, host_permission::{HostAccessPermission}, reservation_map::ReserverKey}, reception_permission::{ReceptionAccessPermission}}, registry_results::{OwnedRegistryAccessResult, OwnedRegistryReplaceResult}};

pub struct OwnedRegistry<
    AccessId, 
    ReserverId,
    Access,
    ResourceId,
    KeyId,
    Resource,
> {
    sync: parking_lot::Mutex<()>,
    registry: ManagedRegistry<ResourceId, Resource>,
    reception: Reception<AccessId, ReserverId, Access, ResourceId, KeyId>
}

impl<
    ReserverId: ReserverKey,
    Access: Accessor<StoredResource = Resource>,
    ResourceId: ResourceKey + AccessKey + Clone,
    KeyId: Key,
    Resource,
> OwnedRegistry<ResourceId, ReserverId, Access, ResourceId, KeyId, Resource> {
    pub fn access(
        &self, 
        resource_id: &ResourceId,
        access: &Access,
        reserver_id: Option<&ReserverId>,
        key: Option<&KeyId>,
    ) -> OwnedRegistryAccessResult<Access::AccessResult<'_, Access::Resource>> { 
        let _sync = self.sync.lock();
        match self.reception.permits_access(resource_id, Some(access), reserver_id, key) {
            ReceptionAccessPermission::NoEntry => OwnedRegistryAccessResult::NoEntry,
            ReceptionAccessPermission::Host(HostAccessPermission::ReservationConflict) => OwnedRegistryAccessResult::ReservationConflict,
            ReceptionAccessPermission::Host(HostAccessPermission::AccessMap(AccessPermission::Insert(_))) => unreachable!("Access is Some"),
            ReceptionAccessPermission::Host(HostAccessPermission::AccessMap(AccessPermission::Access(false))) => OwnedRegistryAccessResult::AccessConflict,
            ReceptionAccessPermission::Host(HostAccessPermission::AccessMap(AccessPermission::Access(true))) | 
            ReceptionAccessPermission::Host(HostAccessPermission::AccessMap(AccessPermission::UnknownAccessId)) => {
                match unsafe { self.registry.access(resource_id, access) } {
                    ManagedRegistryAccessResult::ResourceNotFound => OwnedRegistryAccessResult::ResourceNotFound,
                    ManagedRegistryAccessResult::Found(result) => {
                        // self.reception.record_access();
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
        resource: Resource,
    ) -> OwnedRegistryReplaceResult<Resource> {
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
    Resource,
> Default for OwnedRegistry<AccessId, ReserverId, Access, ResourceId, Key, Resource> {
    fn default() -> Self {
        Self {
            sync: parking_lot::Mutex::default(),
            reception: Reception::default(),
            registry: ManagedRegistry::default()
        }
    }
}