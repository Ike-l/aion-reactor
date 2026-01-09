use tracing::{Level, span};

use crate::prelude::{AccessKey, AccessPermission, AccessRemovalResult, Accessor, HostAccessPermission, HostUnReserveResult, Key, ManagedRegistry, ManagedRegistryAccessResult, ManagedRegistryReplacementResult, Reception, ReceptionAccessPermission, ReceptionDeAccessResult, ReceptionUnReserveResult, RegistryAccessPermission, RegistryAccessResult, RegistryDeAccessResult, RegistryReplacementResult, RegistryReservationResult, RegistryUnReserveResult, ReservationMapUnReserveResult, ReserverKey, ResourceKey};

pub mod managed_registry;
pub mod reception;
pub mod registry_results;

pub struct Registry<
    AccessId, 
    ReserverId,
    Access: Accessor,
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
> Registry<ResourceId, ReserverId, Access, ResourceId, KeyId, Box<StoredResource>> {
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
                        ManagedRegistryAccessResult::AccessFailure => RegistryAccessResult::AccessFailure,
                        ManagedRegistryAccessResult::Found(result) => {
                            self.reception.record_access(resource_id, access, reserver_id, key);
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
                    ManagedRegistryReplacementResult::ResourceNotFound => RegistryReplacementResult::ResourceNotFound,
                    ManagedRegistryReplacementResult::AccessFailure => RegistryReplacementResult::AccessFailure,
                    ManagedRegistryReplacementResult::NoOp => RegistryReplacementResult::NoOp,
                    ManagedRegistryReplacementResult::Found(access_result) => {
                        self.reception.record_access(resource_id, access, reserver_id, key);
                        RegistryReplacementResult::Found(access_result)
                    }
                }
            }
        }
    }

    pub unsafe fn deaccess(
        &self,
        resource_id: &ResourceId,
        access: &Access,
        key: Option<&KeyId>
    ) -> RegistryDeAccessResult {
        let _sync = self.sync.lock();
        match self.reception.deaccess(resource_id, access, key) {
            ReceptionDeAccessResult::Ok => RegistryDeAccessResult::Ok,
            ReceptionDeAccessResult::UnknownAccessId => RegistryDeAccessResult::UnknownResourceId,
            ReceptionDeAccessResult::NoEntry => RegistryDeAccessResult::NoEntry,
        }
    }

    // Reservations must conserve the invariant that a successful reservation *guarantees* access in future
    pub fn reserve(
        &self,
        reserver_id: ReserverId,
        resource_id: ResourceId,
        access: Access,
        key: Option<&KeyId>
    ) -> RegistryReservationResult {
        let _sync = self.sync.lock();
        match self.registry.contains(&resource_id) {
            true => RegistryReservationResult::Reception(self.reception.reserve(reserver_id, resource_id, access, key)),
            false => RegistryReservationResult::NoResource,
        }
    }

    pub fn unreserve(
        &self,
        reserver_id: &ReserverId,
        resource_id: &ResourceId,
        access: &Access,
        key: Option<&KeyId>,
    ) -> RegistryUnReserveResult {
        let _sync = self.sync.lock();
        match self.reception.unreserve(reserver_id, resource_id, access, key) {
            ReceptionUnReserveResult::NoEntry => RegistryUnReserveResult::NoEntry,
            ReceptionUnReserveResult::Host(HostUnReserveResult::ReservationMap(ReservationMapUnReserveResult::NoReservation)) => RegistryUnReserveResult::NoReservation,
            ReceptionUnReserveResult::Host(HostUnReserveResult::ReservationMap(ReservationMapUnReserveResult::AccessMap(AccessRemovalResult::UnknownAccessId))) => RegistryUnReserveResult::UnknownResourceId,
            ReceptionUnReserveResult::Host(HostUnReserveResult::ReservationMap(ReservationMapUnReserveResult::AccessMap(AccessRemovalResult::Split))) => RegistryUnReserveResult::Ok,
        }
    }

    /// Safety:
    /// Make sure all accesses are actually cleared
    pub unsafe fn clear_accesses(&self) {
        self.reception.clear_accesses()
    }
    
}

impl<
    AccessId,
    ReserverId,
    Access: Accessor,
    ResourceId,
    KeyId,
    StoredResource,
> Registry<AccessId, ReserverId, Access, ResourceId, KeyId, StoredResource> {
    pub fn is_active(&self) -> bool {
        self.reception.is_active()
    }
}

impl<
    AccessId,
    ReserverId,
    Access: Accessor,
    ResourceId,
    KeyId,
    StoredResource,
> Drop for Registry<AccessId, ReserverId, Access, ResourceId, KeyId, StoredResource> {
    fn drop(&mut self) {
        if self.is_active() {
            panic!("Tried Dropping Active Registry");
        }
    }
}

impl<
    AccessId,
    ReserverId,
    Access: Accessor,
    ResourceId,
    Key,
    StoredResource,
> Default for Registry<AccessId, ReserverId, Access, ResourceId, Key, Box<StoredResource>> {
    fn default() -> Self {
        Self {
            sync: parking_lot::Mutex::default(),
            reception: Reception::default(),
            registry: ManagedRegistry::default()
        }
    }
}