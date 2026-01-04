pub mod reservation_map;
pub mod access_map;
pub mod host_permission;

use tracing::{Instrument, Level, span};

use crate::prelude::{AccessKey, AccessMap, Accessor, HostAccessPermission, ReservationMap, ReservationMapPermission, ReserverKey};

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
        access: &Access,
    ) -> HostAccessPermission {
        let span = span!(Level::DEBUG, "Host Permits Access");
        let _enter = span.enter();

        match self.reservation_map.permits_access(&reserver_id, &access_id, access) {
            ReservationMapPermission::ReservationConflict(conflicts) => {
                if conflicts {
                    HostAccessPermission::ReservationConflict
                } else {
                    HostAccessPermission::AccessMap(self.access_map.permits_access(&access_id, access))
                }
            },
        }
    }

    pub fn record_access(
        &self,
        access_id: AccessId,
        access: Access,
        reserver_id: Option<&ReserverId>
    ) {
        let span = span!(Level::DEBUG, "Host Record Access");
        let _enter = span.enter();

        if let Some(reserver_id) = reserver_id {
            self.reservation_map.record_access(reserver_id, &access_id, &access);
        }
        self.access_map.record_access(access_id, access);
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