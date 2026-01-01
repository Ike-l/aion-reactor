pub mod reservation_map;
pub mod access_map;
pub mod host_permission;

pub use reservation_map::{
    ReservationMap,
};

pub use access_map::{
    AccessMap,
    Accessor
};

use crate::registry::owned_registry::reception::host::{access_map::{AccessKey}, host_permission::{HostAccessPermission}, reservation_map::{ReserverKey, reservation_map_permission::ReservationMapPermission}};

pub struct Host<
    ReserverId,
    AccessId,
    Access, 
> {
    reservation_map: ReservationMap<ReserverId, AccessId, Access>,
    access_map: AccessMap<AccessId, Access>,
}

impl<
    ReserverId: ReserverKey,
    AccessId: AccessKey,
    Access: Accessor, 
> Host<ReserverId, AccessId, Access> {
    pub fn permits_access(
        &self,
        reserver_id: Option<&ReserverId>,
        access_id: &AccessId,
        access: Option<&Access>,
    ) -> HostAccessPermission {
        if let Some(new_access) = access {
            match self.reservation_map.permits_access(&reserver_id, &access_id, new_access) {
                ReservationMapPermission::ReservationConflict(conflicts) => {
                    if conflicts {
                        HostAccessPermission::ReservationConflict
                    } else {
                        HostAccessPermission::AccessMap(self.access_map.permits_access(&access_id, access))
                    }
                },
            }
        } else {
            HostAccessPermission::AccessMap(self.access_map.permits_access(&access_id, access))
        }
    }
}

impl<
    ReserverId,
    AccessId,
    Access, 
> Default for Host<ReserverId, AccessId, Access> {
    fn default() -> Self {
        Self { 
            reservation_map: ReservationMap::default(), 
            access_map: AccessMap::default() 
        }
    }
}