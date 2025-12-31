pub mod registry;
pub mod reception;
pub mod registry_result;

pub use reception::{
    Reception, 
    Receptionist
};

pub use registry::{
    AdministratedRegistry,
    Administrator
};

use crate::registry::owned_registry::{reception::{Hoster, host::{Accessor, Reserver, access_map::{access_map_permission::AccessMapPermission}, host_permission::HostPermission}, reception_permission::ReceptionPermission}, registry::{RegistryManager, managed_registry::RegistryOperator}, registry_result::RegistryResult};

pub trait Owner {
    type Receptionist: Receptionist;
    type Administrator: Administrator;

    fn resource_id_as_access_id( 
        resource_id: &<<<Self::Administrator as Administrator>::RegistryManager as RegistryManager>::RegistryOperator as RegistryOperator>::ResourceId
    ) -> <<<Self::Receptionist as Receptionist>::Hoster as Hoster>::Accessor as Accessor>::AccessId;
}

pub struct OwnedRegistry<O: Owner> {
    registry: AdministratedRegistry<O::Administrator>,
    reception: Reception<O::Receptionist>
}

impl<O: Owner> OwnedRegistry<O> {
    pub fn get<T: 'static>(
        &self, 
        resource_id: <<<<O as Owner>::Administrator as Administrator>::RegistryManager as RegistryManager>::RegistryOperator as RegistryOperator>::ResourceId,
        reserver_id: Option<&<<<<O as Owner>::Receptionist as Receptionist>::Hoster as Hoster>::Reserver as Reserver>::ReserverId>,
        access: <<<<O as Owner>::Receptionist as Receptionist>::Hoster as Hoster>::Accessor as Accessor>::Access,
    ) -> RegistryResult<'_, T> { 
        let access_id = O::resource_id_as_access_id(&resource_id);

        match self.reception.permits_access(reserver_id, &access_id, &access) {
            ReceptionPermission::Host(host_permission) => {
                match host_permission {
                    HostPermission::AccessMap(access_map_permission) => {
                        match access_map_permission {
                            AccessMapPermission::Coexistence(can_coexist) => {
                                if can_coexist {
                                    self.registry.get(&resource_id, &access)
                                } else {
                                    RegistryResult::AccessMapConflict
                                }
                            },
                            AccessMapPermission::UnknownAccessId => {
                                self.registry.get(&resource_id, &access)
                            },
                        }
                    },
                    HostPermission::ReservationConflict => {
                        RegistryResult::ReservationConflict
                    },
                }
            },
            ReceptionPermission::NoEntry => {
                return RegistryResult::NoEntry
            },
        } 
    }

    // remove has to check *all* accesses, not just conflicts
    // remove has to check reservations the same

    // can reservations be made on a resource that doesnt exist? No!
}

impl<O: Owner> Default for OwnedRegistry<O> {
    fn default() -> Self {
        Self {
            reception: Reception::default(),
            registry: AdministratedRegistry::default()
        }
    }
}