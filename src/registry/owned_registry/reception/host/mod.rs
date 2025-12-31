pub mod reservation_map;
pub mod access_map;
pub mod host_permission;

pub use reservation_map::{
    ReservationMap,
    Reserver
};

pub use access_map::{
    AccessMap,
    Accessor
};

use crate::registry::owned_registry::reception::host::{host_permission::HostPermission, reservation_map::reservation_map_permission::ReservationMapPermission};

pub trait Hoster {
    type Accessor: Accessor;
    type Reserver: Reserver;
}

pub struct Host<H: Hoster> {
    reservation_map: ReservationMap<H::Reserver, H::Accessor>,
    access_map: AccessMap<H::Accessor>,
}

impl<H: Hoster> Host<H> {
    pub fn permits_access(
        &self,
        reserver_id: Option<&<<H as Hoster>::Reserver as Reserver>::ReserverId>,
        access_id: &<<H as Hoster>::Accessor as Accessor>::AccessId,
        access: <<H as Hoster>::Accessor as Accessor>::Access,
    ) -> HostPermission {
        match self.reservation_map.permits_access(&reserver_id, &access_id, &access) {
            ReservationMapPermission::ReservationConflict(conflicts) => {
                if conflicts {
                    HostPermission::ReservationConflict
                } else {
                    HostPermission::AccessMap(self.access_map.permits_access(&access_id, &access))
                }
            },
        }
    }
}

impl<H: Hoster> Default for Host<H> {
    fn default() -> Self {
        Self { 
            reservation_map: ReservationMap::default(), 
            access_map: AccessMap::default() 
        }
    }
}